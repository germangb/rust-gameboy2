import { set_panic_hook, wasm_log_init, is_cgb, GameBoy, Button } from "wasm"

set_panic_hook()
wasm_log_init();

const cgb = is_cgb()
const gb = GameBoy.new()
let lcd_debug_overlay = 0
let paused = true

// html elements
let lcd = document.getElementById("lcd")
let pal = document.getElementById("pal")
let vram0 = document.getElementById("vram0")
let vram1 = document.getElementById("vram1")
let play = document.getElementById("play")
let pause = document.getElementById("pause")
let fullscreen = document.getElementById("fullscreen")
let reset = document.getElementById("reset")
let load = document.getElementById("load")
let file_select = document.getElementById("file_select")
let lcd_tilemap = document.getElementById("lcd_tilemap")
let lcd_window = document.getElementById("lcd_window")
let lcd_sprites = document.getElementById("lcd_sprites")
let lcd_lyc = document.getElementById("lcd_lyc")

// canvas rendering context
let lcd_ctx = lcd.getContext("2d")
let pal_ctx = pal.getContext("2d")
let vram0_ctx = vram0.getContext("2d")
let vram1_ctx = vram1.getContext("2d")

const update_lcd_flags = () => {
    if (lcd_tilemap.checked)
        lcd_debug_overlay |=  0b0001
    else
        lcd_debug_overlay &= ~0b0001
    if (lcd_window.checked)
        lcd_debug_overlay |=  0b0010
    else
        lcd_debug_overlay &= ~0b0010
    if (lcd_sprites.checked)
        lcd_debug_overlay |=  0b0100
    else
        lcd_debug_overlay &= ~0b0100
    if (lcd_lyc.checked)
        lcd_debug_overlay |=  0b1000
    else
        lcd_debug_overlay &= ~0b1000
    gb.set_lcd_overlay_flags(lcd_debug_overlay)
}

update_lcd_flags()

lcd_tilemap.onclick = update_lcd_flags;
lcd_window.onclick = update_lcd_flags;
lcd_sprites.onclick = update_lcd_flags;
lcd_lyc.onclick = update_lcd_flags;

const load_rom = (file) => {
    let reader = new FileReader()
    reader.addEventListener('load', (e) => {
        let buffer = new Uint8Array(e.target.result)
        gb.load_rom(buffer)
        update_lcd_flags()
        do_play()
    })
    reader.readAsArrayBuffer(file)
}

const update_cgb = () => {
    gb.update(lcd_ctx, pal_ctx, vram0_ctx, vram1_ctx)
    if (!paused)
        requestAnimationFrame(update_cgb)
}

const update = () => {
    gb.update(lcd_ctx, pal_ctx, vram0_ctx)
    if (!paused)
        requestAnimationFrame(update)
}

const do_play = () => {
    if (paused) {
        paused = false
        play.disabled = true
        pause.disabled = false

        if (cgb) {
            requestAnimationFrame(update_cgb)
        } else {
            requestAnimationFrame(update)
        }
    }
}

const do_pause = () => {
    if (!paused) {
        paused = true
        play.disabled = false
        pause.disabled = true
    }
}

const do_reset = () => {
    gb.reset()
    update_lcd_flags()
    do_play()
}

const do_fullscreen = () => {
    lcd.requestFullscreen().catch((err) => {
        console.error(err)
    })
}

reset.onclick = do_reset
play.onclick = do_play
pause.onclick = do_pause
load.onclick = () => {
    do_pause()
    file_select.click()
}
file_select.onchange = (e) => {
    load_rom(e.target.files[0])
}
lcd.ondblclick = do_fullscreen
fullscreen.onclick = do_fullscreen

// begin playing
do_play()

document.addEventListener("keydown", (e) => {
    if (e.keyCode == 90)
        gb.press(Button.A)
    if (e.keyCode == 88)
        gb.press(Button.B)
    if (e.keyCode == 13)
        gb.press(Button.Start)
    if (e.keyCode == 16)
        gb.press(Button.Select)
    if (e.keyCode == 37)
        gb.press(Button.Left)
    if (e.keyCode == 39)
        gb.press(Button.Right)
    if (e.keyCode == 40)
        gb.press(Button.Down)
    if (e.keyCode == 38)
        gb.press(Button.Up)
})

document.addEventListener("keyup", (e) => {
    if (e.keyCode == 90)
        gb.release(Button.A)
    if (e.keyCode == 88)
        gb.release(Button.B)
    if (e.keyCode == 13)
        gb.release(Button.Start)
    if (e.keyCode == 16)
        gb.release(Button.Select)
    if (e.keyCode == 37)
        gb.release(Button.Left)
    if (e.keyCode == 39)
        gb.release(Button.Right)
    if (e.keyCode == 40)
        gb.release(Button.Down)
    if (e.keyCode == 38)
        gb.release(Button.Up)
})