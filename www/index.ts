import { Universe } from '../pkg/wasm_pixels'
import { memory } from '../pkg/wasm_pixels_bg';

const num_dots = 100000;

const width = 500;
const height = 500;

let universe = Universe.new(width, height, num_dots);

const canvas = document.getElementById('game-of-life-canvas') as HTMLCanvasElement;
canvas.width = width;
canvas.height = height;

canvas.onclick = event => {
    universe.add_event(event.offsetX, event.offsetY, 100);
}

document.onkeypress = event => {
    console.log(event);
    switch (event.key) {
        case 'r':
            universe.free();
            universe = Universe.new(width, height, num_dots)
            break;
        case 'a':
            universe.tick(10.0);
    }
}

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

    const tickTime = Date.now();

    drawDots();

    const drawTime = Date.now();

    console.log(tickTime - now, drawTime - tickTime);

    requestAnimationFrame(renderLoop);
};

requestAnimationFrame(renderLoop);