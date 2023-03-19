import { Terminal } from 'xterm';

const term = new Terminal();

export function attach() {
    term.open(document.getElementById('terminal_parent'));
}

export function print(str) {
    term.write(str);
}

let uart_data = "";
term.onData((str, _) => {
    uart_data += str;
});

export function read_uart_data() {
    let data = uart_data;
    uart_data = "";
    return data;
}
