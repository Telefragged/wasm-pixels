import { Universe } from '../pkg/wasm_pixels'
import { memory } from '../pkg/wasm_pixels_bg';

const num_dots = 100000;

const width = 600;
const height = 600;

let universe = Universe.new(width, height, num_dots);

const fps = new class {
    fps: HTMLElement;
    frames: number[];
    lastFrameTimeStamp: number;


    constructor() {
        this.fps = document.getElementById("fps");
        this.frames = [];
        this.lastFrameTimeStamp = performance.now();
    }

    render() {
        // Convert the delta time since the last frame render into a measure
        // of frames per second.
        const now = performance.now();
        const delta = now - this.lastFrameTimeStamp;
        this.lastFrameTimeStamp = now;
        const fps = 1 / delta * 1000;

        // Save only the latest 100 timings.
        this.frames.push(fps);
        if (this.frames.length > 100) {
            this.frames.shift();
        }

        // Find the max, min, and mean of our 100 latest timings.
        let min = Infinity;
        let max = -Infinity;
        let sum = 0;
        for (let i = 0; i < this.frames.length; i++) {
            sum += this.frames[i];
            min = Math.min(this.frames[i], min);
            max = Math.max(this.frames[i], max);
        }
        let mean = sum / this.frames.length;

        // Render the statistics.
        this.fps.textContent = `
  Frames per Second:
           latest = ${Math.round(fps)}
  avg of last 100 = ${Math.round(mean)}
  min of last 100 = ${Math.round(min)}
  max of last 100 = ${Math.round(max)}
  `.trim();
    }
};

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

    fps.render();

    const now = Date.now();

    const timeDelta = now - prevTime;

    prevTime = now;

    universe.tick(timeDelta / 1000);

    drawDots();

    remainingDotsLabel.textContent = String(universe.remaining_dots());

    requestAnimationFrame(renderLoop);
};

requestAnimationFrame(renderLoop);