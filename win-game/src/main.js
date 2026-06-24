import { invoke } from '@tauri-apps/api/core';

let pollInterval = null;
let debugInterval = null;
let debugVisible = false;
let ticketAnimActive = false;
let canvas = document.getElementById('fireworksCanvas');
let ctx = canvas.getContext('2d');
let particles = [];
let animFrame = null;

function initCanvas() {
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
}
window.addEventListener('resize', initCanvas);
initCanvas();

class Particle {
  constructor(x, y, color) {
    this.x = x; this.y = y; this.color = color;
    const angle = Math.random() * Math.PI * 2;
    const speed = Math.random() * 6 + 2;
    this.vx = Math.cos(angle) * speed;
    this.vy = Math.sin(angle) * speed;
    this.life = 1;
    this.decay = Math.random() * 0.02 + 0.01;
  }
  update() {
    this.x += this.vx; this.y += this.vy; this.vy += 0.05; this.life -= this.decay;
  }
  draw() {
    ctx.globalAlpha = this.life;
    ctx.fillStyle = this.color;
    ctx.beginPath();
    ctx.arc(this.x, this.y, 3, 0, Math.PI * 2);
    ctx.fill();
    ctx.globalAlpha = 1;
  }
}

function burstFireworks() {
  const colors = ['#ff0', '#f0f', '#0ff', '#ff4444', '#44ff44', '#ffaa00', '#ff0088', '#00ff88'];
  const cx = Math.random() * canvas.width * 0.6 + canvas.width * 0.2;
  const cy = Math.random() * canvas.height * 0.4 + canvas.height * 0.1;
  for (let i = 0; i < 80; i++) {
    particles.push(new Particle(cx, cy, colors[Math.floor(Math.random() * colors.length)]));
  }
}

function animateFireworks() {
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  particles = particles.filter(p => p.life > 0);
  particles.forEach(p => { p.update(); p.draw(); });
  if (particles.length > 0) {
    animFrame = requestAnimationFrame(animateFireworks);
  } else {
    canvas.style.display = 'none';
  }
}

function startFireworks() {
  canvas.style.display = 'block';
  for (let i = 0; i < 5; i++) setTimeout(() => burstFireworks(), i * 400);
  if (animFrame) cancelAnimationFrame(animFrame);
  animateFireworks();
}

function spawnTicketAnimation(finalTickets) {
  if (ticketAnimActive) return;
  ticketAnimActive = true;
  const ticketOut = document.getElementById('ticketOut');
  const ticketVal = document.getElementById('ticketVal');
  let current = finalTickets;
  const interval = setInterval(() => {
    current = Math.max(0, current - Math.ceil(current / 8) - 1);
    ticketVal.textContent = current;
    const ticket = document.createElement('div');
    ticket.className = 'ticket-falling';
    ticket.style.left = Math.random() * 80 + 10 + '%';
    ticket.style.animationDuration = (1.5 + Math.random()) + 's';
    ticketOut.appendChild(ticket);
    setTimeout(() => ticket.remove(), 3000);
    if (current <= 0) {
      clearInterval(interval);
      ticketVal.textContent = '0';
      setTimeout(() => {
        startFireworks();
        const popup = document.createElement('div');
        popup.className = 'score-popup';
        popup.textContent = `WINNER! ${finalTickets} TICKETS!`;
        popup.style.left = '50%'; popup.style.top = '40%';
        popup.style.transform = 'translateX(-50%)';
        document.body.appendChild(popup);
        setTimeout(() => popup.remove(), 3000);
      }, 500);
      setTimeout(() => { ticketAnimActive = false; }, 4000);
    }
  }, 250);
}

function updateScoreList(scores) {
  const list = document.getElementById('scoreList');
  list.innerHTML = '<div class="score-row header"><span>#</span><span>INITIALS</span><span>SCORE</span><span>TICKETS</span></div>';
  scores.forEach((s, i) => {
    const row = document.createElement('div');
    row.className = 'score-row' + (i === 0 ? ' top1' : i === 1 ? ' top2' : i === 2 ? ' top3' : '');
    row.innerHTML = `<span>${i+1}</span><span>${s.initials}</span><span>${s.score.toLocaleString()}</span><span>${s.tickets.toLocaleString()}</span>`;
    list.appendChild(row);
  });
}

function updateLamp(id, active) {
  const el = document.getElementById(id);
  if (el) el.classList.toggle('active', active);
}

function setCabinetDimmed(dimmed) {
  document.getElementById('arcadeView').classList.toggle('waiting', dimmed);
}

function setGameName(name) {
  const el = document.getElementById('gameName');
  if (el) el.textContent = name || '---';
}

async function updateDisplay() {
  let connected = false;
  let gameName = '';
  try {
    const status = await invoke('get_status');
    connected = status.connected;
    gameName = status.game_name;
    setGameName(gameName);
  } catch (_) { connected = false; }

  const connStatus = document.getElementById('connStatus');
  if (connected) {
    connStatus.textContent = 'Connected';
    connStatus.className = 'connection-status connected';
  } else {
    connStatus.textContent = 'Waiting';
    connStatus.className = 'connection-status';
  }
  setCabinetDimmed(!connected);

  if (!connected) {
    document.getElementById('coinVal').textContent = '0';
    document.getElementById('jackpotVal').textContent = '0';
    document.getElementById('hsVal').textContent = '0';
    document.getElementById('ticketVal').textContent = '0';
    updateLamp('lampStart', false);
    updateLamp('lampLeader', false);
    document.getElementById('billboardLed').classList.remove('active');
    document.getElementById('marqueeLed').classList.remove('active');
    document.getElementById('sideLeds').querySelectorAll('.led-strip').forEach(el => el.classList.remove('active'));
    document.getElementById('wooferLed').classList.remove('active');
    document.getElementById('itemLed').classList.remove('active');
    return;
  }

  // Connected — update display from outputs
  document.getElementById('coinVal').textContent = (await invoke('get_outputs')).coin1;
  document.getElementById('jackpotVal').textContent = (await invoke('get_outputs')).ticket_jackpot;
  document.getElementById('hsVal').textContent = (await invoke('get_outputs')).high_score;
  document.getElementById('ticketVal').textContent = (await invoke('get_outputs')).ticket_counter;

  const o = await invoke('get_outputs');
  updateLamp('lampStart', o.lamps.LampStart);
  updateLamp('lampLeader', o.lamps.LampLeader);
  document.getElementById('billboardLed').classList.toggle('active',
    o.lamps['Billboard Red'] || o.lamps['Billboard Green'] || o.lamps['Billboard Blue']);
  document.getElementById('marqueeLed').classList.toggle('active',
    o.lamps.LampRed || o.lamps.LampGreen || o.lamps.LampBlue);
  document.getElementById('sideLeds').querySelectorAll('.led-strip').forEach(el => el.classList.toggle('active',
    o.lamps.SideLEDRed || o.lamps.SideLEDGreen || o.lamps.SideLEDBlue));
  document.getElementById('wooferLed').classList.toggle('active',
    o.lamps.WooferLEDRed || o.lamps.WooferLEDGreen || o.lamps.WooferLEDBlue);
  document.getElementById('itemLed').classList.toggle('active',
    o.lamps.ItemLEDRed || o.lamps.ItemLEDGreen || o.lamps.ItemLEDBlue);

  updateScoreList(await invoke('get_scores'));

  const roundData = await invoke('round_ended');
  if (roundData) {
    const [score, tickets] = roundData;
    document.getElementById('modalScore').textContent = score.toLocaleString();
    document.getElementById('modalTickets').textContent = tickets.toLocaleString();
    document.getElementById('initialsModal').style.display = 'flex';
    document.getElementById('initialsInput').value = '';
    document.getElementById('initialsInput').focus();
    spawnTicketAnimation(tickets);
  }
}

// --- Debug overlay ---
async function updateDebugLog() {
  if (!debugVisible) return;
  try {
    const logs = await invoke('get_logs');
    const container = document.getElementById('debugLog');
    container.innerHTML = logs.map(l => {
      const cls = l.startsWith('[ERROR]') ? 'log-error' : 'log-info';
      return `<div class="${cls}">${l}</div>`;
    }).join('');
    container.scrollTop = container.scrollHeight;
  } catch (_) {}
}

document.addEventListener('keydown', (e) => {
  if (e.key === 'F12') {
    e.preventDefault();
    debugVisible = !debugVisible;
    document.getElementById('debugOverlay').style.display = debugVisible ? 'flex' : 'none';
    if (debugVisible) updateDebugLog();
  }
});

// --- Submit initials ---
document.getElementById('submitBtn').addEventListener('click', async () => {
  const input = document.getElementById('initialsInput');
  const initials = input.value.toUpperCase().slice(0, 3).padEnd(3, ' ');
  const score = parseInt(document.getElementById('modalScore').textContent.replace(/,/g,'')) || 0;
  const tickets = parseInt(document.getElementById('modalTickets').textContent.replace(/,/g,'')) || 0;
  if (initials.trim()) updateScoreList(await invoke('submit_score', { initials, score, tickets }));
  document.getElementById('initialsModal').style.display = 'none';
});
document.getElementById('initialsInput').addEventListener('keydown', (e) => {
  if (e.key === 'Enter') document.getElementById('submitBtn').click();
});
document.getElementById('initialsInput').addEventListener('input', (e) => {
  e.target.value = e.target.value.toUpperCase().replace(/[^A-Z]/g,'').slice(0,3);
});

// --- Close button ---
document.getElementById('closeBtn').addEventListener('click', () => {
  invoke('close_app');
});

// --- Simulate button ---
document.getElementById('simulateBtn').addEventListener('click', async () => {
  document.getElementById('simulateBtn').textContent = 'Simulating...';
  await invoke('simulate');
  await updateDisplay();
  document.getElementById('simulateBtn').textContent = 'Sim Data';
});

// --- Start ---
pollInterval = setInterval(updateDisplay, 200);
debugInterval = setInterval(updateDebugLog, 500);
updateDisplay();
