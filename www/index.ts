import { Universe } from '../pkg/wasm_pixels'
import { memory } from '../pkg/wasm_pixels_bg';

const num_dots = 100;

const universe = Universe.new(800, 800, num_dots);

// Give the canvas room for all of our cells and a 1px border
// around each of them.
const canvas = document.getElementById('game-of-life-canvas') as HTMLCanvasElement;
canvas.height = 800;
canvas.width = 800;

const ctx = canvas.getContext('2d');

const drawDots = () => {
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    const dotsPtr = universe.dots();
    const dots = new Float64Array(memory.buffer, dotsPtr, num_dots * 2)

    for (let i = 0; i < num_dots; i++) {
        const x = dots[i * 2];
        const y = dots[i * 2 + 1];

        ctx.fillRect(x-1,y-1,3,3);
    }
};

let prevTime = Date.now();

const renderLoop = () => {

    const now = Date.now();

    const timeDelta = now - prevTime;

    prevTime = now;

    universe.tick(timeDelta / 100);

    // drawGrid();
    drawDots();

    requestAnimationFrame(renderLoop);
};

drawDots();
requestAnimationFrame(renderLoop);