import init, { apply_event } from './wasm/mortal_treasures_world.js';

document.addEventListener('DOMContentLoaded', () => {
    console.log('[load] Document loaded.');
    main()
});

async function main() {
    await init();
    console.log('[load] WASM loaded.');

    const counter_el = document.getElementById('counter');
    const increment_el = document.getElementById('increment');
    const decrement_el = document.getElementById('decrement');
    const text_el = document.getElementById('text');
    const game_over_el = document.getElementById('game_over');

    let world = { count: 5, text: "hello???" };

    const display = () => {
        counter_el.innerText = world.count.toString();
        text_el.value = world.text;
        console.log(`[state] world: ${JSON.stringify(world)}`);
    };
    const handle_event = (e) => {
        world = apply_event(world, e);
        display();
    }

    const socket = new WebSocket("ws://127.0.0.1:3000/ws");

    socket.onopen = function (e) {
        console.log("[open] Connection established");

        increment_el.onclick = () => socket.send(JSON.stringify({ kind: 'Increment' }));
        decrement_el.onclick = () => socket.send(JSON.stringify({ kind: 'Decrement' }));
        text_el.onchange = function () {
            let s = this.value;
            display();
            socket.send(JSON.stringify({ kind: 'Text', s }));
        };
        game_over_el.onclick = () => socket.send(JSON.stringify({ kind: 'GameOver' }));
    };

    socket.onmessage = function (event) {
        let m = event.data;
        console.log(`[message] Data received from server: ${m}`);
        let v = JSON.parse(m);
        handle_event(v);
    };

    socket.onclose = function (event) {
        if (event.wasClean) {
            console.log(`[close] Connection closed cleanly, code=${event.code} reason=${event.reason}`);
        } else {
            // e.g. server process killed or network down
            // event.code is usually 1006 in this case
            console.log('[close] Connection died');
        }
    };

    socket.onerror = function (error) {
        console.error(`[error]`);
    };
}
