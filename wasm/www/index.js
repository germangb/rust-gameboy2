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

    if (e.keyCode == 90) gb.press(Button.A)
    if (e.keyCode == 88) gb.press(Button.B)
    if (e.keyCode == 13) gb.press(Button.Start)
    if (e.keyCode == 16) gb.press(Button.Select)
    if (e.keyCode == 37) gb.press(Button.Left)
    if (e.keyCode == 39) gb.press(Button.Right)
    if (e.keyCode == 40) gb.press(Button.Down)
    if (e.keyCode == 38) gb.press(Button.Up)
})

document.addEventListener("keyup", (e) => {
    if (e.keyCode == 90) gb.release(Button.A)
    if (e.keyCode == 88) gb.release(Button.B)
    if (e.keyCode == 13) gb.release(Button.Start)
    if (e.keyCode == 16) gb.release(Button.Select)
    if (e.keyCode == 37) gb.release(Button.Left)
    if (e.keyCode == 39) gb.release(Button.Right)
    if (e.keyCode == 40) gb.release(Button.Down)
    if (e.keyCode == 38) gb.release(Button.Up)
})
