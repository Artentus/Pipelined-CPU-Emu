import Clusterize from "clusterize.js";
import * as jam1emu from "@artentus/jam1emu";

const frameRate = 59.94047619047765; // VGA 60Hz
const millisecondsPerFrame = 1000 / frameRate;

const screenWidth = 640;
const screenHeight = 480;

const terminalParent = document.getElementById('terminal_parent');
const mainRight = document.getElementById('main_right');
const canvas = document.getElementById('canvas');
const ctx2d = canvas.getContext('2d');
const system = jam1emu.System.create();

const filePicker = document.getElementById("file_picker");
const runButton = document.getElementById("run_button");
const singleStepButton = document.getElementById("single_step_button");
const frameStepButton = document.getElementById("frame_step_button");
const resetButton = document.getElementById("reset_button");
const regs16BitText = document.getElementById("regs_16_bit_text");
const regs8BitText = document.getElementById("regs_8_bit_text");
const flagsText = document.getElementById("flags_text");

let running = false;
let currentTime = undefined;
let elapsed = 0;

function update_vga() {
    let framebuffer = system.framebuffer();
    let image = new ImageData(framebuffer, screenWidth, screenHeight);
    ctx2d.putImageData(image, 0, 0);
}

function update_16_bit_regs() {
    const pc = system.pc().toString(16).toUpperCase().padStart(4, "0");
    const ra = system.ra().toString(16).toUpperCase().padStart(4, "0");
    const sp = system.sp().toString(16).toUpperCase().padStart(4, "0");
    const si = system.si().toString(16).toUpperCase().padStart(4, "0");
    const di = system.di().toString(16).toUpperCase().padStart(4, "0");
    const tx = system.tx().toString(16).toUpperCase().padStart(4, "0");
    regs16BitText.innerText = `PC: ${pc}\nRA: ${ra}\nSP: ${sp}\nSI: ${si}\nDI: ${di}\nTX: ${tx}`;
}

function update_8_bit_regs() {
    const a = system.a().toString(16).toUpperCase().padStart(2, "0");
    const b = system.b().toString(16).toUpperCase().padStart(2, "0");
    const c = system.c().toString(16).toUpperCase().padStart(2, "0");
    const d = system.d().toString(16).toUpperCase().padStart(2, "0");
    const tl = system.tl().toString(16).toUpperCase().padStart(2, "0");
    const th = system.th().toString(16).toUpperCase().padStart(2, "0");
    regs8BitText.innerText = `A:  ${a}\nB:  ${b}\nC:  ${c}\nD:  ${d}\nTL: ${tl}\nTH: ${th}`;
}

function update_flags() {
    const flags = system.flags();
    const f = (flags & 0x20) !== 0 ? "1" : "0";
    const l = (flags & 0x10) !== 0 ? "1" : "0";
    const c = (flags & 0x08) !== 0 ? "1" : "0";
    const z = (flags & 0x04) !== 0 ? "1" : "0";
    const s = (flags & 0x02) !== 0 ? "1" : "0";
    const o = (flags & 0x01) !== 0 ? "1" : "0";
    flagsText.innerText = `${f} ${l} ${c} ${z} ${s} ${o}`;
}

const memory_clusterize = new Clusterize({
    rows: [],
    scrollId: "memory_scroll_area",
    contentId: "memory_content_area"
});

function update_memory_view() {
    const memoryView = system.memory_view();

    let lines = [];
    for (let addr = 0; addr <= 0xFFFF; addr += 16) {
        let str = "<div style='margin: 0px 8px'>"
        str += addr.toString(16).toUpperCase().padStart(4, "0");
        str += " |";

        for (let offset = 0; offset < 16; offset++) {
            str += " ";
            str += memoryView[addr + offset].toString(16).toUpperCase().padStart(2, "0");
        }

        str += "</div>";
        lines.push(str);
    }

    memory_clusterize.update(lines);
}

function update() {
    update_vga();
    update_16_bit_regs();
    update_8_bit_regs();
    update_flags();
    update_memory_view();
}

function toggle_run() {
    running = !running;
    elapsed = 0;
    runButton.innerText = running ? "Pause" : "Run";
    filePicker.disabled = running;
    singleStepButton.disabled = running;
    frameStepButton.disabled = running;
    update();
}

function pause() {
    running = false;
    elapsed = 0;
    runButton.innerText = "Run";
    filePicker.disabled = false;
    singleStepButton.disabled = false;
    frameStepButton.disabled = false;
    update();
}

canvas.width = screenWidth;
canvas.height = screenHeight;
system.reset();
update();

filePicker.onchange = pe => {
    if (!running) {
        let file = pe.target.files[0];
        if (file !== null) {
            var reader = new FileReader();
            reader.readAsArrayBuffer(file);

            reader.onload = re => {
                let content = re.target.result;
                let bytes = new Uint8Array(content);
                system.load_program(bytes);
            };
        }
    }
};

runButton.onclick = () => {
    toggle_run();
};

singleStepButton.onclick = () => {
    if (!running) {
        system.clock(1);
        update();
    }
};

frameStepButton.onclick = () => {
    if (!running) {
        system.clock_frame();
        update();
    }
};

resetButton.onclick = () => {
    pause();
    system.reset();
};

function renderLoop(now) {
    if (currentTime === undefined) {
        currentTime = now;
    }

    if (running) {
        elapsed += now - currentTime;

        if ((elapsed / millisecondsPerFrame) >= 5) {
            // Lag, don't try to catch up because that lags even more.
            elapsed = 0;
        }

        const needs_update = elapsed >= millisecondsPerFrame;

        while (elapsed >= millisecondsPerFrame) {
            elapsed -= millisecondsPerFrame;

            let breakPoint = system.clock_frame();
            if (breakPoint) {
                pause();
            }
        }

        if (needs_update) {
            update();
        }
    }

    // Position the VGA output correctly.
    let canvas_width = window.innerWidth - mainRight.offsetWidth - 16;
    let canvas_height = Math.max(200, window.innerHeight - terminalParent.clientHeight - 16);
    let scaleX = canvas_width / screenWidth;
    let scaleY = canvas_height / screenHeight;
    let scale = Math.min(scaleX, scaleY);
    let offsetX = (canvas_width - (screenWidth * scale)) / 2;
    let offsetY = (canvas_height - (screenHeight * scale)) / 2;
    canvas.style.transform = `translate(${offsetX}px, ${offsetY}px) scale(${scale}, ${scale})`;

    currentTime = now;
    requestAnimationFrame(renderLoop);
};

requestAnimationFrame(renderLoop);
