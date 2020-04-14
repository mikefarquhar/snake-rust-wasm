import init, { Game, Direction, UpdateResult } from './pkg/snake.js'

// Module-wide state. Will be initialised asynchronously.
let game
let boardWidth
let boardHeight

let memoryView

let tileset
const tileSize = 8 // px

let ctx

let scoreElement
let highScore

// Simple wrapper for async/await image loading.
function loadImage (src) {
    return new Promise((resolve, reject) => {
        const image = new Image()
        image.onload = () => resolve(image)
        image.onerror = () => reject(new Error(`Failed to load image from src: ${src}`))
        image.src = src
    })
}

// Render the game
function render () {
    // Update the score
    scoreElement.innerText = `Score: ${game.get_score()}, High Score: ${highScore}`
    // clear the screen
    ctx.fillStyle = '#FFFFFF'
    ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height)

    // render the snake.
    const snakeLength = game.get_snake_len()

    let tileX = 0
    let tileY = 0

    const { boardX: headX, boardY: headY, direction: headDir } = extractSnakePart(0)
    const { boardX: tailX, boardY: tailY } = extractSnakePart(snakeLength - 1)

    // Render the snake body first
    let lastX = headX
    let lastY = headY
    let lastDir = headDir

    for (let i = 1; i < snakeLength; ++i) {
        const { boardX, boardY, direction } = extractSnakePart(i)

        // Skip parts that have the same cell index (the apple travelling down the body).
        // Additionally don't render if it has the same position as the tail.
        if ((boardX === lastX && boardY === lastY) || (boardX === tailX && boardY === tailY)) {
            continue
        }

        if (
            (direction === Direction.Up && lastDir === Direction.Up) ||
            (direction === Direction.Down && lastDir === Direction.Down)
        ) {
            tileX = 0
            tileY = 3
        } else if (
            (direction === Direction.Right && lastDir === Direction.Right) ||
            (direction === Direction.Left && lastDir === Direction.Left)
        ) {
            tileX = 1
            tileY = 3
        } else if (
            (direction === Direction.Left && lastDir === Direction.Up) ||
            (direction === Direction.Down && lastDir === Direction.Right)
        ) {
            tileX = 0
            tileY = 2
        } else if (
            (direction === Direction.Up && lastDir === Direction.Right) ||
            (direction === Direction.Left && lastDir === Direction.Down)
        ) {
            tileX = 1
            tileY = 2
        } else if (
            (direction === Direction.Up && lastDir === Direction.Left) ||
            (direction === Direction.Right && lastDir === Direction.Down)
        ) {
            tileX = 2
            tileY = 2
        } else {
            tileX = 3
            tileY = 2
        }

        renderTile(tileX, tileY, boardX, boardY)

        lastDir = direction
        lastX = boardX
        lastY = boardY
    }

    // Render the snake tail
    tileY = 1

    switch (lastDir) {
        case Direction.Up:
            tileX = 0
            break
        case Direction.Right:
            tileX = 1
            break
        case Direction.Down:
            tileX = 2
            break
        case Direction.Left:
            tileX = 3
            break
    }

    renderTile(tileX, tileY, tailX, tailY)

    // Finally render the snake head.
    tileY = 0

    switch (headDir) {
        case Direction.Up:
            tileX = 0
            break
        case Direction.Right:
            tileX = 1
            break
        case Direction.Down:
            tileX = 2
            break
        case Direction.Left:
            tileX = 3
            break
    }

    renderTile(tileX, tileY, headX, headY)

    // render the apple
    const appleCellIndex = game.get_apple()
    if (appleCellIndex != null) {
        const appleX = appleCellIndex % boardWidth
        const appleY = (appleCellIndex - appleX) / boardWidth
        renderTile(3, 3, appleX, appleY)
    }
}

function extractSnakePart (n) {
    const ptr = game.get_snake_part(n)
    const cellIndex = memoryView.getUint8(ptr)
    const boardX = cellIndex % boardWidth
    const boardY = (cellIndex - boardX) / boardWidth
    const direction = memoryView.getUint8(ptr + 1)
    return { boardX, boardY, direction }
}

function renderTile (tileX, tileY, boardX, boardY) {
    ctx.drawImage(
        tileset,
        tileX * tileSize,
        tileY * tileSize,
        tileSize,
        tileSize,
        boardX * tileSize,
        boardY * tileSize,
        tileSize,
        tileSize
    )
}

// State when the game is ready to run, but is waiting to be started.
function setWaitingState () {
    function keyListener (evt) {
        const keys = ['ArrowUp', 'ArrowLeft', 'ArrowDown', 'ArrowRight']
        if (keys.includes(evt.key)) {
            window.removeEventListener('keydown', keyListener)
            setRunningState(evt)
        }
    }

    game.reset()
    render()
    window.addEventListener('keydown', keyListener)
}

// State while the game is running.
function setRunningState (initialKeyEvent) {
    // Initialise the game update system (all times are in milliseconds).
    const dt = 1000 / 5.5
    let accum = 0
    let prevT = performance.now()

    // Player input. Input is buffered for better game feel.
    const pressed = []

    // Updates the game at a frequency of 1000 / dt ticks per second.
    function update (t) {
        accum += t - prevT
        prevT = t
        if (accum >= dt) {
            accum -= dt
            const result = game.update(pressed.shift())
            render()

            if (result === UpdateResult.GameOver) {
                window.removeEventListener('keydown', keyListener)
                setGameOverState()
                return
            }
        }

        window.requestAnimationFrame(update)
    }

    function keyListener (evt) {
        // Only buffer 2 inputs max.
        if (pressed.length > 2) { return }

        let nextPressed
        switch (evt.key) {
            case 'ArrowUp':
                nextPressed = Direction.Up
                break
            case 'ArrowLeft':
                nextPressed = Direction.Left
                break
            case 'ArrowDown':
                nextPressed = Direction.Down
                break
            case 'ArrowRight':
                nextPressed = Direction.Right
                break
            default:
                return // Don't do anything for other keys.
        }

        pressed.push(nextPressed)
    }

    keyListener(initialKeyEvent)
    window.addEventListener('keydown', keyListener)
    window.requestAnimationFrame(update)
}

function setGameOverState () {
    const score = game.get_score()
    if (score > highScore) {
        highScore = score
        localStorage.setItem('snakeHighScore', highScore)
    }

    ctx.fillStyle = '#FF0000'
    ctx.fillRect(0, canvas.height / 2 - 10, canvas.width, 20)

    ctx.fillStyle = '#FFFFFF'
    ctx.textAlign = 'center'
    ctx.fillText('Game over', canvas.width / 2, canvas.height / 2 + 3)

    function keyListener (evt) {
        window.removeEventListener('keydown', keyListener)
        setWaitingState()
    }

    window.addEventListener('keydown', keyListener)
}

// IIFE for running the code with async/await support.
(async function () {
    const [{ memory }, image] = await Promise.all([init(), loadImage('./snake.png')])

    // Set the module-wide state
    game = new Game()
    boardWidth = game.get_width()
    boardHeight = game.get_height()

    memoryView = new DataView(memory.buffer)

    tileset = image

    const canvas = document.querySelector('#canvas')
    canvas.width = tileSize * boardWidth
    canvas.height = tileSize * boardHeight
    ctx = canvas.getContext('2d')

    scoreElement = document.querySelector('#score')
    highScore = Number.parseInt(localStorage.getItem('snakeHighScore')) || 0

    // Set the initial game state
    setWaitingState()
})()