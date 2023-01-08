import { createApp } from 'https://unpkg.com/vue@3/dist/vue.global.js';
import init, { apply_event } from './wasm/mortal_treasures_world.js';

document.addEventListener('DOMContentLoaded', () => {
    console.log('[load] Document loaded.');
    main()
});

async function main() {
    await init();
    console.log('[load] WASM loaded.');

    app().mount('#app');
}

function app() {
    createApp({
        data() {
            return {
                world: {
                    count: 5,
                    text: 'hello???',
                },
                socket: null,
            }
        },
        methods: {
            increment() {
                socket.send(JSON.stringify({ kind: 'Increment' }));
            },
            decrement() {
                socket.send(JSON.stringify({ kind: 'Decrement' }));
            },
            set_text(e) {
                socket.send(JSON.stringify({ kind: 'Text', s: e.value }));
            },
            game_over() {
                socket.send(JSON.stringify({ kind: 'GameOver' }));
            }
        },
        mounted() {
            const socket = new WebSocket("ws://127.0.0.1:3000/ws");

            socket.onmessage = (event) => {
                let m = event.data;
                console.log(`[message] Data received from server: ${m}`);
                let v = JSON.parse(m);
                this.world = apply_event(this.world, v);
            };


            socket.onclose = (event) => {
                if (event.wasClean) {
                    console.log(`[close] Connection closed cleanly, code=${event.code} reason=${event.reason}`);
                } else {
                    // e.g. server process killed or network down
                    // event.code is usually 1006 in this case
                    console.log('[close] Connection died');
                }
            };

            socket.onerror = () => console.error(`[error]`);

            this.socket = socket;
        }
    })
}
