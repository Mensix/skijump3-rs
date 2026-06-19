import init, { start } from '../pkg/skijump3.js';

await init();

const canvas = document.getElementById('game');
canvas.focus();
canvas.addEventListener('click', () => canvas.focus());

start('game');
