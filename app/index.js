import Clusterize from "clusterize.js";
import * as jam1emu from "@artentus/jam1emu";
import { Terminal } from 'xterm';
import * as ace from 'ace-builds/src-noconflict/ace'

require("ace-builds/src-noconflict/theme-monokai");

ace.define(
    "ace/mode/jam1asm_highlight_rules",
    ["require", "exports", "module", "ace/lib/oop", "ace/mode/text_highlight_rules"],
    function(require, exports, module) {
        "use strict";

        var oop = require("../lib/oop");
        var TextHighlightRules = require("./text_highlight_rules").TextHighlightRules;

        var Jam1AsmHighlightRules = function() {
            this.$rules = {
                start: [
                    {
                        token: 'keyword.directive.jam1asm',
                        regex: '\\.[a-zA-Z_][a-zA-Z0-9_]*',
                    },
                    {
                        token: 'keyword.instruction.jam1asm',
                        regex: '\\b((nop)|(mov)|(inc)|(incc)|(dec)|(in)|(out)|(break)|(lodsb)|(stosb)|(call)|(ret)|(callbd)|(retbd)|(jmp)|(jo)|(jno)|(js)|(jns)|(jz)|(jnz)|(je)|(jne)|(jc)|(jnc)|(jnae)|(jb)|(jae)|(jnb)|(jbe)|(jna)|(ja)|(jnbe)|(jl)|(jnge)|(jge)|(jnl)|(jle)|(jng)|(jg)|(jnle)|(jlc)|(jnlc)|(push)|(pop)|(clc)|(shl)|(shr)|(add)|(addc)|(addac)|(sub)|(subb)|(subae)|(and)|(or)|(xor)|(not)|(cmp)|(test))\\b',
                        caseInsensitive: true
                    },
                    {
                        token: 'constant.numeric.jam1asm',
                        regex: '\\b([0-9][a-zA-Z0-9_]*)\\b',
                    },
                    {
                        token: 'string.quoted.double.jam1asm',
                        regex: '"',
                        push: [
                            {
                                token: 'constant.language.escape.jam1asm',
                                regex: '\\\\(x[0-9a-fA-F]{2}|u[0-9a-fA-F]{4}|.)',
                            },
                            {
                                token: 'string.quoted.double.jam1asm',
                                regex: '"',
                                next: 'pop'
                            },
                            {
                                token: 'string.quoted.double.jam1asm',
                                regex: '$',
                                next: 'pop'
                            },
                            {
                                token: 'string.quoted.double.jam1asm',
                                regex: '.',
                            },
                        ]
                    },
                    {
                        token: 'comment.line.jam1asm',
                        regex: '//|\\;',
                        push: [
                            {
                                token: 'comment.line.jam1asm',
                                regex: '$',
                                next: 'pop'
                            },
                            {
                                token: 'comment.line.jam1asm',
                                regex: '.',
                            },
                        ]
                    },
                ]
            };
            this.normalizeRules();
        };

        Jam1AsmHighlightRules.metaData = {
            fileTypes: ['asm', 'inc'],
            name: 'jam1asm',
            scopeName: 'source.jam1asm'
        };

        oop.inherits(Jam1AsmHighlightRules, TextHighlightRules);
        exports.Jam1AsmHighlightRules = Jam1AsmHighlightRules;
    }
);

ace.define(
    "ace/mode/jam1asm",
    ["require", "exports", "module", "ace/lib/oop", "ace/mode/text", "ace/mode/jam1asm_highlight_rules"],
    function(require, exports, module) {
        "use strict";

        var oop = require("../lib/oop");
        var TextMode = require("./text").Mode;
        var Jam1AsmHighlightRules = require("./jam1asm_highlight_rules").Jam1AsmHighlightRules;

        //var FoldMode = require("./folding/coffee").FoldMode;

        var Mode = function () {
            this.HighlightRules = Jam1AsmHighlightRules;
            //this.foldingRules = new FoldMode();
            this.$behaviour = this.$defaultBehaviour;
        };

        oop.inherits(Mode, TextMode);

        (function () {
            this.lineCommentStart = ["//", ";"];
            this.$id = "ace/mode/jam1asm";
        }).call(Mode.prototype);
        exports.Mode = Mode;
    }
);                

(function() {
    ace.require(["ace/mode/jam1asm"], function(m) {
        if (typeof module == "object" && typeof exports == "object" && module) {
            module.exports = m;
        }
    });
})();

const codeEditor = ace.edit("code_editor");
codeEditor.setTheme("ace/theme/monokai");
codeEditor.session.setMode("ace/mode/jam1asm");

const term = new Terminal({ scrollback: 10000, cols: 60, rows: 20 });
term.open(document.getElementById('output_terminal_parent'));

const frameRate = 59.94047619047765; // VGA 60Hz
const millisecondsPerFrame = 1000 / frameRate;

const screenWidth = 640;
const screenHeight = 480;

const terminalParent = document.getElementById('terminal_parent');
const mainRight = document.getElementById('main_right');
const controls = document.getElementById('controls');
const registerView = document.getElementById('register_view');
const memoryView = document.getElementById('memory_view');
const memoryScrollArea = document.getElementById('memory_scroll_area');
const mainEditor = document.getElementById('main_editor');
const assemblerOutput = document.getElementById("assembler_output");
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
const assembleButton = document.getElementById("assemble_button");

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

const memoryClusterize = new Clusterize({
    rows: [],
    scrollId: "memory_scroll_area",
    contentId: "memory_content_area",
    rows_in_block: 100,
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

    memoryClusterize.update(lines);
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
    assembleButton.disabled = running;
    update();
}

function pause() {
    running = false;
    elapsed = 0;
    runButton.innerText = "Run";
    filePicker.disabled = false;
    singleStepButton.disabled = false;
    frameStepButton.disabled = false;
    assembleButton.disabled = false;
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
                if (system.load_program(0, bytes)) {
                    update_memory_view();
                } else {
                    // TODO:
                }
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

assembleButton.onclick = () => {
    if (!running) {
        term.clear();
        let output = system.assemble(codeEditor.getValue());
        term.write(output);
        update_memory_view();
    }
};

function resize() {
    memoryScrollArea.style.height = Math.max(200, (window.innerHeight - memoryView.offsetHeight - registerView.offsetHeight - controls.offsetHeight - 10)) + "px";
    document.getElementById("code_editor").style.height = Math.max(200, (document.documentElement.clientHeight - assemblerOutput.clientHeight - 16)) + "px";

    // Position the VGA output correctly.
    let canvas_width = Math.max(screenWidth, document.documentElement.clientWidth - mainRight.offsetWidth - mainEditor.offsetWidth - 16);
    let canvas_height = Math.max(screenHeight, document.documentElement.clientHeight - terminalParent.clientHeight - 16);
    let scaleX = canvas_width / screenWidth;
    let scaleY = canvas_height / screenHeight;
    let scale = Math.min(scaleX, scaleY);
    let offsetX = (canvas_width - (screenWidth * scale)) / 2;
    let offsetY = (canvas_height - (screenHeight * scale)) / 2;
    canvas.style.transform = `translate(${offsetX}px, ${offsetY}px) scale(${scale}, ${scale})`;
}

onresize = () => {
    resize();
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

    currentTime = now;
    requestAnimationFrame(renderLoop);
};

resize();
requestAnimationFrame(renderLoop);
