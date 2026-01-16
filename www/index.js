import { Universe } from "game-of-life";

const CELL_SIZE = 3; // Size of each cell in pixels
const GRID_COLOR = "#000000";

// Generate 25 distinct colors using HSL color space
function generateColors(count) {
  const colors = [];
  for (let i = 0; i < count; i++) {
    const hue = (i * 360 / count) % 360;
    const saturation = 70 + (i % 3) * 10;
    const lightness = 50 + (i % 2) * 10;

    // Convert HSL to RGB
    const rgb = hslToRgb(hue / 360, saturation / 100, lightness / 100);
    colors.push(rgb);
  }
  return colors;
}

function hslToRgb(h, s, l) {
  let r, g, b;

  if (s === 0) {
    r = g = b = l;
  } else {
    const hue2rgb = (p, q, t) => {
      if (t < 0) t += 1;
      if (t > 1) t -= 1;
      if (t < 1/6) return p + (q - p) * 6 * t;
      if (t < 1/2) return q;
      if (t < 2/3) return p + (q - p) * (2/3 - t) * 6;
      return p;
    };

    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    r = hue2rgb(p, q, h + 1/3);
    g = hue2rgb(p, q, h);
    b = hue2rgb(p, q, h - 1/3);
  }

  return [Math.round(r * 255), Math.round(g * 255), Math.round(b * 255)];
}

// Start the game (WASM is auto-initialized)
{
  // Setup canvas
  const canvas = document.getElementById("game-of-life-canvas");
  const ctx = canvas.getContext("2d");

  // Calculate grid dimensions based on window size
  // Make canvas large enough that the circular view is always filled when rotating
  const viewportDiagonal = Math.sqrt(
    window.innerWidth * window.innerWidth +
    window.innerHeight * window.innerHeight
  );

  // Canvas needs to be at least diagonal size to ensure the circle is always filled
  const canvasSize = Math.ceil(viewportDiagonal * 1.5);
  const width = Math.floor(canvasSize / CELL_SIZE);
  const height = Math.floor(canvasSize / CELL_SIZE);

  canvas.width = width * CELL_SIZE;
  canvas.height = height * CELL_SIZE;

  // Center the canvas in the viewport
  canvas.style.position = 'absolute';
  canvas.style.left = '50%';
  canvas.style.top = '50%';
  canvas.style.transform = 'translate(-50%, -50%)';

  const GHOST_SIZE = 60; // Size to render the ghost

  // Function to draw a Pac-Man style ghost
  const drawGhost = (x, y) => {
    const centerX = x + GHOST_SIZE / 2;
    const centerY = y + GHOST_SIZE / 2;
    const radius = GHOST_SIZE / 2;

    ctx.save();

    // Draw ghost body (rounded top, wavy bottom)
    ctx.fillStyle = 'rgba(255, 100, 100, 0.8)'; // Semi-transparent red
    ctx.beginPath();

    // Top half circle
    ctx.arc(centerX, centerY, radius, Math.PI, 0, false);

    // Wavy bottom
    const waveCount = 3;
    const waveWidth = (GHOST_SIZE) / waveCount;
    for (let i = 0; i < waveCount; i++) {
      const waveX = x + i * waveWidth;
      ctx.lineTo(waveX + waveWidth / 2, y + GHOST_SIZE - 5);
      ctx.lineTo(waveX + waveWidth, y + GHOST_SIZE);
    }

    ctx.closePath();
    ctx.fill();

    // Draw eyes
    const eyeRadius = radius / 5;
    const eyeOffsetX = radius / 2.5;
    const eyeOffsetY = radius / 3;

    // Left eye
    ctx.fillStyle = 'white';
    ctx.beginPath();
    ctx.arc(centerX - eyeOffsetX, centerY - eyeOffsetY, eyeRadius, 0, Math.PI * 2);
    ctx.fill();

    // Right eye
    ctx.beginPath();
    ctx.arc(centerX + eyeOffsetX, centerY - eyeOffsetY, eyeRadius, 0, Math.PI * 2);
    ctx.fill();

    // Left pupil
    ctx.fillStyle = 'black';
    ctx.beginPath();
    ctx.arc(centerX - eyeOffsetX, centerY - eyeOffsetY, eyeRadius / 2, 0, Math.PI * 2);
    ctx.fill();

    // Right pupil
    ctx.beginPath();
    ctx.arc(centerX + eyeOffsetX, centerY - eyeOffsetY, eyeRadius / 2, 0, Math.PI * 2);
    ctx.fill();

    ctx.restore();
  };

  // Initialize universe
  const universe = Universe.new(width, height);

  // Generate 25 distinct colors and seed the universe in a circle
  const colors = generateColors(25);

  // Pack colors as u32 (0x00RRGGBB format) for passing to Rust
  const packedColors = colors.map(color => {
    return (color[0] << 16) | (color[1] << 8) | color[2];
  });

  // Initialize with 25 nodes in a circle
  universe.seed_circle(new Uint32Array(packedColors));

  let rotationAngle = 0;
  const rotationSpeed = (2 * Math.PI) / (60 * 60); // Full rotation in 60 seconds (assuming 60 fps)

  let zoomLevel = 1.0;
  const MIN_ZOOM = 0.5;
  const MAX_ZOOM = 3.0;
  const ZOOM_STEP = 0.1;

  // Mouse wheel zoom
  canvas.addEventListener('wheel', (e) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? -ZOOM_STEP : ZOOM_STEP;
    zoomLevel = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, zoomLevel + delta));
  });

  // Create zoom control buttons
  const zoomControls = document.createElement('div');
  zoomControls.style.position = 'fixed';
  zoomControls.style.top = '20px';
  zoomControls.style.right = '20px';
  zoomControls.style.zIndex = '1000';
  zoomControls.style.display = 'flex';
  zoomControls.style.flexDirection = 'column';
  zoomControls.style.gap = '10px';

  const createButton = (text, onClick) => {
    const btn = document.createElement('button');
    btn.textContent = text;
    btn.style.width = '40px';
    btn.style.height = '40px';
    btn.style.fontSize = '24px';
    btn.style.border = '2px solid #666';
    btn.style.borderRadius = '5px';
    btn.style.background = 'rgba(0, 0, 0, 0.7)';
    btn.style.color = '#fff';
    btn.style.cursor = 'pointer';
    btn.style.userSelect = 'none';
    btn.addEventListener('click', onClick);
    btn.addEventListener('mouseenter', () => btn.style.background = 'rgba(50, 50, 50, 0.9)');
    btn.addEventListener('mouseleave', () => btn.style.background = 'rgba(0, 0, 0, 0.7)');
    return btn;
  };

  const zoomInBtn = createButton('+', () => {
    zoomLevel = Math.min(MAX_ZOOM, zoomLevel + ZOOM_STEP);
  });

  const zoomOutBtn = createButton('-', () => {
    zoomLevel = Math.max(MIN_ZOOM, zoomLevel - ZOOM_STEP);
  });

  zoomControls.appendChild(zoomInBtn);
  zoomControls.appendChild(zoomOutBtn);
  document.body.appendChild(zoomControls);

  const drawCells = () => {
    // Clear the entire canvas
    ctx.save();
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.fillStyle = GRID_COLOR;
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.restore();

    // Apply zoom and rotation around center
    ctx.save();
    ctx.translate(canvas.width / 2, canvas.height / 2);
    ctx.scale(zoomLevel, zoomLevel);
    ctx.rotate(rotationAngle);
    ctx.translate(-canvas.width / 2, -canvas.height / 2);

    for (let row = 0; row < height; row++) {
      for (let col = 0; col < width; col++) {
        const cellData = universe.get_cell(row, col);

        if (cellData !== 0) {
          const r = (cellData >> 16) & 0xFF;
          const g = (cellData >> 8) & 0xFF;
          const b = cellData & 0xFF;

          ctx.fillStyle = `rgb(${r}, ${g}, ${b})`;
          ctx.fillRect(
            col * CELL_SIZE,
            row * CELL_SIZE,
            CELL_SIZE,
            CELL_SIZE
          );
        }
      }
    }

    // Draw ghost if active
    if (universe.is_ghost_active()) {
      const ghostX = universe.ghost_x() * CELL_SIZE - GHOST_SIZE / 2;
      const ghostY = universe.ghost_y() * CELL_SIZE - GHOST_SIZE / 2;
      drawGhost(ghostX, ghostY);
    }

    ctx.restore(); // Restore rotation

    // Update rotation angle
    rotationAngle += rotationSpeed;
  };

  let animationId = null;

  const renderLoop = () => {
    universe.tick();
    drawCells();

    animationId = requestAnimationFrame(renderLoop);
  };

  // Initial draw
  drawCells();

  // Start animation
  renderLoop();
}
