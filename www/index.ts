import { Universe } from '../pkg/wasm_pixels'
import { memory } from '../pkg/wasm_pixels_bg';

const num_dots = 100000;

const width = 600;
const height = 600;

let universe = Universe.new(width, height, num_dots);

const canvas = document.getElementById('game-canvas') as HTMLCanvasElement;
canvas.width = width;
canvas.height = height;

canvas.onclick = event => {
    universe.add_event(event.offsetX, event.offsetY, 10);
}

const numberOfDotsInput = document.getElementById('number-of-dots-input') as HTMLInputElement;

numberOfDotsInput.valueAsNumber = num_dots;

const resetGameButton = document.getElementById('reset-game-button') as HTMLButtonElement;

resetGameButton.onclick = _ => {
    universe.free();
    universe = Universe.new(width, height, Math.max(numberOfDotsInput.valueAsNumber, 1))
}

const remainingDotsLabel = document.getElementById('remaining-dots') as HTMLParagraphElement

const ctx = canvas.getContext('2d');

ctx.imageSmoothingEnabled = false;

const drawDots = () => {
    universe.render_image_data();

    const imagePtr = universe.image_data();
    const imageArray = new Uint8ClampedArray(memory.buffer, imagePtr, width * height * 4);
    
    const imageData = new ImageData(imageArray, width, height);

    ctx.putImageData(imageData, 0, 0);
};

let prevTime = Date.now();

const renderLoop = () => {

    const now = Date.now();

    const timeDelta = now - prevTime;

    prevTime = now;

    universe.tick(timeDelta / 1000);

    drawDots();

    remainingDotsLabel.textContent = String(universe.remaining_dots());

    requestAnimationFrame(renderLoop);
};

requestAnimationFrame(renderLoop);