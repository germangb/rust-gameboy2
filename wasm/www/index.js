import { GameBoy, Button, set_panic_hook } from "wasm";

set_panic_hook()

let debug = false

const gb = GameBoy.new()
const canvas = document.getElementById("canvas")
const ctx = canvas.getContext("2d")

ctx.fillStyle = 'green';
//ctx.fillRect(10, 10, 150, 100);

const draw = () => {
    gb.update_frame(ctx)
    requestAnimationFrame(draw)
}

requestAnimationFrame(draw)

document.addEventListener("keydown", (e) => {
    if (e.keyCode == 68) {
        debug = !debug;
        gb.set_debug_overlays(debug)
    }
})

document.addEventListener("keyup", (e) => {
    //console.log(e)
})
