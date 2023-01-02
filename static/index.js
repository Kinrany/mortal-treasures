document.addEventListener('DOMContentLoaded', () => {
    console.log('[load] Document loaded.');

    const counter_el = document.getElementById('counter');
    const increment_el = document.getElementById('increment');
    const decrement_el = document.getElementById('decrement');
    const exit_el = document.getElementById('exit');

    let world = { count: 5 };
    const display = () => {
        counter_el.innerText = world.count.toString();
        console.log(`[state] world: ${JSON.stringify(world)}`);
    };
    const handle_event = (e) => {
        switch (e.kind) {
            case 'World':
                delete e.kind;
                world = e;
                break;
            case 'Increment':
                world.count += 1;
                break;
            case 'Decrement':
                world.count -= 1;
                break;
            default:
                console.log(`[error] Unknown event kind: ${e.kind}`);
        }
        display();
    }

    const socket = new WebSocket("ws://127.0.0.1:3000/ws");

    socket.onopen = function (e) {
        console.log("[open] Connection established");

        increment_el.onclick = () => socket.send(JSON.stringify({ kind: 'Increment' }));
        decrement_el.onclick = () => socket.send(JSON.stringify({ kind: 'Decrement' }));
        exit_el.onclick = () => socket.close();
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
});