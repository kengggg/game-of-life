use wasm_bindgen::prelude::*;
use js_sys::Math;
use std::f64::consts::PI;

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    tick_count: u32,
    colors: Vec<(u8, u8, u8)>,
    comet_timer: u32,
    ghost_timer: u32,
    ghost_active: bool,
    ghost_position: i32,
    ghost_direction: u32, // 0=left-right, 1=right-left, 2=top-bottom, 3=bottom-top
    ghost_lane: i32,
    ghost_speed: i32, // pixels per tick
}

#[derive(Clone, Copy)]
struct Cell {
    alive: bool,
    r: u8,
    g: u8,
    b: u8,
}

impl Cell {
    fn new() -> Cell {
        Cell {
            alive: false,
            r: 0,
            g: 0,
            b: 0,
        }
    }

    fn with_color(r: u8, g: u8, b: u8) -> Cell {
        Cell {
            alive: true,
            r,
            g,
            b,
        }
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn new(width: u32, height: u32) -> Universe {
        let cells = vec![Cell::new(); (width * height) as usize];

        Universe {
            width,
            height,
            cells,
            tick_count: 0,
            colors: Vec::new(),
            comet_timer: 0,
            ghost_timer: 0,
            ghost_active: false,
            ghost_position: 0,
            ghost_direction: 0,
            ghost_lane: 0,
            ghost_speed: 0,
        }
    }

    pub fn seed_circle(&mut self, colors: Vec<u32>) {
        // Store colors for later respawning
        self.colors = colors.iter().map(|&c| {
            let r = ((c >> 16) & 0xFF) as u8;
            let g = ((c >> 8) & 0xFF) as u8;
            let b = (c & 0xFF) as u8;
            (r, g, b)
        }).collect();

        let center_x = self.width as f64 / 2.0;
        let center_y = self.height as f64 / 2.0;
        let radius = (self.width.min(self.height) as f64 / 4.0);

        // Place 25 cells in a circle, each as a small cluster for stability
        for i in 0..25 {
            let angle = (i as f64) * 2.0 * PI / 25.0;
            let center_cell_x = (center_x + radius * angle.cos()) as u32;
            let center_cell_y = (center_y + radius * angle.sin()) as u32;
            let (r, g, b) = self.colors[i % self.colors.len()];

            // Create a small cluster (3x3) around each point for better survival
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let x = (center_cell_x as i32 + dx) as u32;
                    let y = (center_cell_y as i32 + dy) as u32;

                    if x < self.width && y < self.height {
                        let idx = self.get_index(x, y);
                        self.cells[idx] = Cell::with_color(r, g, b);
                    }
                }
            }
        }
    }

    pub fn seed_person(&mut self, r: u8, g: u8, b: u8, cell_count: u32) {
        for _ in 0..cell_count {
            let x = (Math::random() * self.width as f64) as u32;
            let y = (Math::random() * self.height as f64) as u32;
            let idx = self.get_index(x, y);
            self.cells[idx] = Cell::with_color(r, g, b);
        }
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(col, row);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(col, row);

                let next_cell = match (cell.alive, live_neighbors) {
                    // Rule 1: Any live cell with 2 or 3 live neighbors survives
                    (true, 2) | (true, 3) => cell,
                    // Rule 2: Any dead cell with exactly 3 live neighbors becomes alive
                    (false, 3) => {
                        let blended_color = self.blend_neighbor_colors(col, row);
                        Cell::with_color(blended_color.0, blended_color.1, blended_color.2)
                    }
                    // Rule 3: All other cells die or stay dead
                    _ => Cell::new(),
                };

                next[idx] = next_cell;
            }
        }

        self.cells = next;
        self.tick_count += 1;
        self.comet_timer += 1;
        self.ghost_timer += 1;

        // Spawn new life every 5 ticks to keep the scene very lively and filled
        if self.tick_count % 5 == 0 && !self.colors.is_empty() {
            self.spawn_random_life();
        }

        // Spawn a comet drop every 120-180 ticks (dramatic effect)
        let comet_interval = 120 + (Math::random() * 60.0) as u32;
        if self.comet_timer >= comet_interval {
            self.spawn_comet_drop();
            self.comet_timer = 0;
        }

        // Spawn a ghost sweep every 600-1200 ticks (10-20 seconds)
        let ghost_interval = 600 + (Math::random() * 600.0) as u32;
        if self.ghost_timer >= ghost_interval && !self.ghost_active {
            self.spawn_ghost();
            self.ghost_timer = 0;
        }

        // Move the ghost if active
        if self.ghost_active {
            self.move_ghost();
        }
    }

    pub fn get_cell(&self, row: u32, col: u32) -> u32 {
        let idx = self.get_index(col, row);
        let cell = self.cells[idx];
        if cell.alive {
            // Pack RGB into a u32: 0x00RRGGBB
            ((cell.r as u32) << 16) | ((cell.g as u32) << 8) | (cell.b as u32)
        } else {
            0
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn is_ghost_active(&self) -> bool {
        self.ghost_active
    }

    pub fn ghost_x(&self) -> i32 {
        match self.ghost_direction {
            0 | 1 => self.ghost_position,
            _ => self.ghost_lane,
        }
    }

    pub fn ghost_y(&self) -> i32 {
        match self.ghost_direction {
            0 | 1 => self.ghost_lane,
            _ => self.ghost_position,
        }
    }
}

impl Universe {
    fn get_index(&self, col: u32, row: u32) -> usize {
        (row * self.width + col) as usize
    }

    fn live_neighbor_count(&self, col: u32, row: u32) -> u8 {
        let mut count = 0;

        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (col + delta_col) % self.width;
                let idx = self.get_index(neighbor_col, neighbor_row);

                if self.cells[idx].alive {
                    count += 1;
                }
            }
        }

        count
    }

    fn blend_neighbor_colors(&self, col: u32, row: u32) -> (u8, u8, u8) {
        let mut live_neighbors = Vec::new();

        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (col + delta_col) % self.width;
                let idx = self.get_index(neighbor_col, neighbor_row);

                if self.cells[idx].alive {
                    live_neighbors.push(self.cells[idx]);
                }
            }
        }

        // Average the RGB values of all live neighbors
        if live_neighbors.is_empty() {
            return (255, 255, 255);
        }

        let r_sum: u32 = live_neighbors.iter().map(|c| c.r as u32).sum();
        let g_sum: u32 = live_neighbors.iter().map(|c| c.g as u32).sum();
        let b_sum: u32 = live_neighbors.iter().map(|c| c.b as u32).sum();
        let count = live_neighbors.len() as u32;

        (
            (r_sum / count) as u8,
            (g_sum / count) as u8,
            (b_sum / count) as u8,
        )
    }

    fn spawn_random_life(&mut self) {
        // Spawn 20-30 random cell clusters to fill the scene
        let spawn_count = 20 + (Math::random() * 11.0) as u32;

        for _ in 0..spawn_count {
            let center_x = (Math::random() * self.width as f64) as u32;
            let center_y = (Math::random() * self.height as f64) as u32;

            // Pick a random color from the stored colors
            let color_idx = (Math::random() * self.colors.len() as f64) as usize;
            let (r, g, b) = self.colors[color_idx];

            // Spawn a 3x3 cluster for better coverage
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let x = (center_x as i32 + dx) as u32;
                    let y = (center_y as i32 + dy) as u32;

                    if x < self.width && y < self.height {
                        let idx = self.get_index(x, y);
                        self.cells[idx] = Cell::with_color(r, g, b);
                    }
                }
            }
        }
    }

    fn spawn_comet_drop(&mut self) {
        // Generate a completely new random color for this comet
        let r = (Math::random() * 256.0) as u8;
        let g = (Math::random() * 256.0) as u8;
        let b = (Math::random() * 256.0) as u8;

        // Random impact point
        let impact_x = (Math::random() * self.width as f64) as i32;
        let impact_y = (Math::random() * self.height as f64) as i32;

        // Create 3D ripple effect with 5 concentric circles
        let num_ripples = 5;

        for ripple in 0..num_ripples {
            let radius = 3.0 + (ripple as f64 * 4.0); // Expanding radius
            let density = 1.0 / (1.0 + ripple as f64 * 0.3); // Decreasing density for depth
            let num_points = (24.0 * (ripple as f64 + 1.0)) as i32; // More points on outer rings

            for point in 0..num_points {
                // Only spawn cells based on density (gives sparse outer rings)
                if Math::random() > density {
                    continue;
                }

                let angle = (point as f64) * 2.0 * PI / (num_points as f64);
                let x = impact_x + (radius * angle.cos()) as i32;
                let y = impact_y + (radius * angle.sin()) as i32;

                // Check bounds
                if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                    let idx = self.get_index(x as u32, y as u32);

                    // Brighten center, darken edges for 3D depth effect
                    let depth_factor = 1.0 - (ripple as f64 / num_ripples as f64) * 0.4;
                    let r_adjusted = (r as f64 * depth_factor) as u8;
                    let g_adjusted = (g as f64 * depth_factor) as u8;
                    let b_adjusted = (b as f64 * depth_factor) as u8;

                    self.cells[idx] = Cell::with_color(r_adjusted, g_adjusted, b_adjusted);

                    // Add some fill in the ripples (creates the wave effect)
                    if ripple > 0 && Math::random() > 0.7 {
                        let fill_radius = radius - 2.0;
                        let fill_x = impact_x + (fill_radius * angle.cos()) as i32;
                        let fill_y = impact_y + (fill_radius * angle.sin()) as i32;

                        if fill_x >= 0 && fill_x < self.width as i32
                            && fill_y >= 0 && fill_y < self.height as i32 {
                            let fill_idx = self.get_index(fill_x as u32, fill_y as u32);
                            self.cells[fill_idx] = Cell::with_color(r_adjusted, g_adjusted, b_adjusted);
                        }
                    }
                }
            }
        }

        // Add a bright center impact point with cluster
        for dy in -2..=2 {
            for dx in -2..=2 {
                let x = impact_x + dx;
                let y = impact_y + dy;

                if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                    let idx = self.get_index(x as u32, y as u32);
                    self.cells[idx] = Cell::with_color(r, g, b);
                }
            }
        }
    }

    fn spawn_ghost(&mut self) {
        // Initialize a new ghost sweep
        self.ghost_active = true;
        self.ghost_direction = (Math::random() * 4.0) as u32;

        // Calculate speed: traverse screen in 4-8 seconds (240-480 ticks at 60fps)
        let duration_ticks = 240 + (Math::random() * 240.0) as i32;

        match self.ghost_direction {
            0 | 1 => {
                // Horizontal movement
                self.ghost_speed = (self.width as i32) / duration_ticks;
                if self.ghost_speed == 0 { self.ghost_speed = 1; }
                self.ghost_lane = (Math::random() * self.height as f64) as i32;
                self.ghost_position = if self.ghost_direction == 0 { 0 } else { self.width as i32 - 1 };
            }
            _ => {
                // Vertical movement
                self.ghost_speed = (self.height as i32) / duration_ticks;
                if self.ghost_speed == 0 { self.ghost_speed = 1; }
                self.ghost_lane = (Math::random() * self.width as f64) as i32;
                self.ghost_position = if self.ghost_direction == 2 { 0 } else { self.height as i32 - 1 };
            }
        }
    }

    fn move_ghost(&mut self) {
        let ghost_width = 20; // Width of the ghost's clearing path

        match self.ghost_direction {
            0 => {
                // Left to right
                for offset in -(ghost_width as i32)..=(ghost_width as i32) {
                    let y = self.ghost_lane + offset;
                    if self.ghost_position >= 0 && self.ghost_position < self.width as i32
                        && y >= 0 && y < self.height as i32 {
                        let idx = self.get_index(self.ghost_position as u32, y as u32);
                        self.cells[idx] = Cell::new();
                    }
                }
                self.ghost_position += self.ghost_speed;
                if self.ghost_position >= self.width as i32 {
                    self.ghost_active = false;
                }
            }
            1 => {
                // Right to left
                for offset in -(ghost_width as i32)..=(ghost_width as i32) {
                    let y = self.ghost_lane + offset;
                    if self.ghost_position >= 0 && self.ghost_position < self.width as i32
                        && y >= 0 && y < self.height as i32 {
                        let idx = self.get_index(self.ghost_position as u32, y as u32);
                        self.cells[idx] = Cell::new();
                    }
                }
                self.ghost_position -= self.ghost_speed;
                if self.ghost_position < 0 {
                    self.ghost_active = false;
                }
            }
            2 => {
                // Top to bottom
                for offset in -(ghost_width as i32)..=(ghost_width as i32) {
                    let x = self.ghost_lane + offset;
                    if x >= 0 && x < self.width as i32
                        && self.ghost_position >= 0 && self.ghost_position < self.height as i32 {
                        let idx = self.get_index(x as u32, self.ghost_position as u32);
                        self.cells[idx] = Cell::new();
                    }
                }
                self.ghost_position += self.ghost_speed;
                if self.ghost_position >= self.height as i32 {
                    self.ghost_active = false;
                }
            }
            _ => {
                // Bottom to top
                for offset in -(ghost_width as i32)..=(ghost_width as i32) {
                    let x = self.ghost_lane + offset;
                    if x >= 0 && x < self.width as i32
                        && self.ghost_position >= 0 && self.ghost_position < self.height as i32 {
                        let idx = self.get_index(x as u32, self.ghost_position as u32);
                        self.cells[idx] = Cell::new();
                    }
                }
                self.ghost_position -= self.ghost_speed;
                if self.ghost_position < 0 {
                    self.ghost_active = false;
                }
            }
        }
    }
}
