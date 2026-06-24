import './styles.css';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

let pollInterval = null;
let debugInterval = null;
let debugVisible = false;
let ticketAnimActive = false;
let canvas = document.getElementById('fireworksCanvas');
let ctx = canvas.getContext('2d');
let particles = [];
let animFrame = null;
let roundEndPending = false;

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
  if (ticketAnimActive || finalTickets <= 0) return;
  ticketAnimActive = true;
  const ticketOut = document.getElementById('ticketOut');
  const ticketVal = document.getElementById('ticketVal');
  let current = finalTickets;
  const interval = setInterval(() => {
    current = Math.max(0, current - Math.ceil(current / 8) - 1);
    ticketVal.textContent = current;
    for (let i = 0; i < 3; i++) {
      const ticket = document.createElement('div');
      ticket.className = 'ticket-falling';
      ticket.style.left = Math.random() * 80 + 10 + '%';
      ticket.style.animationDuration = (1.5 + Math.random()) + 's';
      ticketOut.appendChild(ticket);
      setTimeout(() => ticket.remove(), 3000);
    }
    if (current <= 0) {
      clearInterval(interval);
      ticketVal.textContent = '0';
      setTimeout(() => {
        startFireworks();
        const popup = document.createElement('div');
        popup.className = 'score-popup';
        popup.textContent = `${finalTickets} TICKETS!`;
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

function categoryColor(name, value) {
  if (!value || value === '0') return null;
  if (name.includes('Red')) return '#ff2244';
  if (name.includes('Green')) return '#22ff44';
  if (name.includes('Blue')) return '#2288ff';
  if (name.includes('Yellow')) return '#ffdd00';
  if (name.includes('White')) return '#ffffff';
  if (name.includes('Cyan')) return '#00ffff';
  if (name.includes('Magneta') || name.includes('Magenta')) return '#ff00ff';
  if (name.includes('Orange')) return '#ff8800';
  if (name.includes('Purple')) return '#aa44ff';
  return '#888888';
}

function updateLED(containerId, name, active, color) {
  const container = document.getElementById(containerId);
  if (!container) return;
  let led = container.querySelector(`[data-name="${name}"]`);
  if (!led) {
    led = document.createElement('span');
    led.className = 'color-led';
    led.dataset.name = name;
    led.title = name;
    container.appendChild(led);
  }
  if (active) {
    led.style.background = color || categoryColor(name, '1');
    led.style.boxShadow = `0 0 12px ${color || categoryColor(name, '1')}`;
    led.classList.add('on');
  } else {
    led.style.background = '#1a1a3a';
    led.style.boxShadow = 'none';
    led.classList.remove('on');
  }
}

function buildMiscBox(outputs) {
  const container = document.getElementById('miscOut');
  if (!container) return;
  const known = new Set([
    'LampStart','LampLeader','LampRed','LampGreen','LampBlue',
    'Billboard Red','Billboard Green','Billboard Blue',
    'WooferLEDRed','WooferLEDGreen','WooferLEDBlue',
    'SideLEDRed','SideLEDGreen','SideLEDBlue',
    'ItemLEDRed','ItemLEDGreen','ItemLEDBlue',
    'TicketCounter','TicketJackpot','Coin1','Coin2','HighScore','Rings',
    'pause','mame_start','mame_stop',
  ]);
  container.innerHTML = '';
  for (const [name, val] of Object.entries(outputs)) {
    if (known.has(name) || val === '0' || val === '0') continue;
    const led = document.createElement('span');
    led.className = 'color-led misc-led';
    led.title = `${name} = ${val}`;
    const c = categoryColor(name, val);
    led.style.background = c || '#888';
    if (c) led.style.boxShadow = `0 0 8px ${c}`;
    led.classList.add('on');
    container.appendChild(led);
  }
  if (!container.children.length) {
    container.innerHTML = '<span class="misc-empty">—</span>';
  }
}

async function updateDisplay() {
  let connected = false;
  let gameName = '';
  try {
    const status = await invoke('get_status');
    connected = status.connected;
    gameName = status.game_name;
    document.getElementById('gameName').textContent = gameName || '---';
  } catch (_) { connected = false; }

  const connStatus = document.getElementById('connStatus');
  const statusBar = document.getElementById('statusBar');
  if (connected) {
    connStatus.textContent = 'Connected';
    connStatus.className = 'connection-status connected';
    statusBar.textContent = gameName ? `${gameName} — Connected` : 'Connected';
    statusBar.className = 'status-bar connected';
  } else {
    connStatus.textContent = 'Waiting';
    connStatus.className = 'connection-status';
    statusBar.textContent = 'Waiting for game (port 37520)...';
    statusBar.className = 'status-bar';
  }
  document.getElementById('arcadeView').classList.toggle('waiting', !connected);

  if (!connected) {
    document.getElementById('coinVal').textContent = '0';
    document.getElementById('jackpotVal').textContent = '0';
    document.getElementById('hsVal').textContent = '0';
    document.getElementById('ringsVal').textContent = '0';
    document.getElementById('ticketVal').textContent = '0';
    document.querySelectorAll('.color-led').forEach(el => { el.style.background = '#1a1a3a'; el.style.boxShadow = 'none'; el.classList.remove('on'); });
    return;
  }

  const o = await invoke('get_outputs');
  document.getElementById('coinVal').textContent = o.coin1;
  document.getElementById('jackpotVal').textContent = o.ticket_jackpot;
  document.getElementById('hsVal').textContent = o.high_score;
  document.getElementById('ringsVal').textContent = o.rings;
  document.getElementById('ticketVal').textContent = o.ticket_counter;

  // Billboard
  updateLED('billboardOut', 'Billboard Red', o.lamps['Billboard Red'], '#ff2244');
  updateLED('billboardOut', 'Billboard Green', o.lamps['Billboard Green'], '#22ff44');
  updateLED('billboardOut', 'Billboard Blue', o.lamps['Billboard Blue'], '#2288ff');
  // Billboard triangle color
  const bShape = document.getElementById('billboardShape');
  if (bShape) {
    bShape.classList.remove('active-red','active-green','active-blue','active-mixed');
    const r = o.lamps['Billboard Red'];
    const g = o.lamps['Billboard Green'];
    const b = o.lamps['Billboard Blue'];
    const activeCount = [r,g,b].filter(Boolean).length;
    if (activeCount >= 2) bShape.classList.add('active-mixed');
    else if (r) bShape.classList.add('active-red');
    else if (g) bShape.classList.add('active-green');
    else if (b) bShape.classList.add('active-blue');
  }

  // Lamps
  updateLED('lampsOut', 'LampStart', o.lamps.LampStart, '#00ff44');
  updateLED('lampsOut', 'LampLeader', o.lamps.LampLeader, '#ffaa00');
  updateLED('lampsOut', 'LampRed', o.lamps.LampRed, '#ff2244');
  updateLED('lampsOut', 'LampGreen', o.lamps.LampGreen, '#22ff44');
  updateLED('lampsOut', 'LampBlue', o.lamps.LampBlue, '#2288ff');

  // Woofers
  updateLED('wooferOut', 'WooferLEDRed', o.lamps.WooferLEDRed, '#ff2244');
  updateLED('wooferOut', 'WooferLEDGreen', o.lamps.WooferLEDGreen, '#22ff44');
  updateLED('wooferOut', 'WooferLEDBlue', o.lamps.WooferLEDBlue, '#2288ff');
  // Woofer speaker glow
  document.querySelectorAll('.woofer').forEach(el => {
    el.classList.toggle('active', o.lamps.WooferLEDRed || o.lamps.WooferLEDGreen || o.lamps.WooferLEDBlue);
  });

  // Side LEDs
  updateLED('sideLeftOut', 'SideLEDRed', o.lamps.SideLEDRed, '#ff2244');
  updateLED('sideLeftOut', 'SideLEDGreen', o.lamps.SideLEDGreen, '#22ff44');
  updateLED('sideLeftOut', 'SideLEDBlue', o.lamps.SideLEDBlue, '#2288ff');
  updateLED('sideRightOut', 'SideLEDRed', o.lamps.SideLEDRed, '#ff2244');
  updateLED('sideRightOut', 'SideLEDGreen', o.lamps.SideLEDGreen, '#22ff44');
  updateLED('sideRightOut', 'SideLEDBlue', o.lamps.SideLEDBlue, '#2288ff');

  // Item LEDs
  updateLED('itemOut', 'ItemLEDRed', o.lamps.ItemLEDRed, '#ff2244');
  updateLED('itemOut', 'ItemLEDGreen', o.lamps.ItemLEDGreen, '#22ff44');
  updateLED('itemOut', 'ItemLEDBlue', o.lamps.ItemLEDBlue, '#2288ff');

  // Marquee bar
  const marqueeBar = document.getElementById('marqueeBar');
  if (marqueeBar) {
    marqueeBar.classList.toggle('active', o.lamps.LampRed || o.lamps.LampGreen || o.lamps.LampBlue);
  }

  // Misc - any output not in our layout, showing correct color
  buildMiscBox(o.raw);

  updateScoreList(await invoke('get_scores'));

  // Round end detection
  if (!roundEndPending) {
    const roundData = await invoke('round_ended');
    if (roundData) {
      roundEndPending = true;
      const [score, tickets] = roundData;
      document.getElementById('modalScore').textContent = score.toLocaleString();
      document.getElementById('modalTickets').textContent = tickets.toLocaleString();
      document.getElementById('initialsModal').style.display = 'flex';
      document.getElementById('initialsInput').value = '';
      document.getElementById('initialsInput').focus();
      spawnTicketAnimation(tickets);
    }
  }

  // Check coins = 0 for initials prompt
  const coinVal = parseInt(o.coin1) + parseInt(o.coin2);
  if (coinVal === 0 && connected) {
    document.getElementById('coinsExhausted').style.display = 'block';
  } else {
    document.getElementById('coinsExhausted').style.display = 'none';
  }
}

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
  if (e.key === 'Escape') {
    document.getElementById('initialsModal').style.display = 'none';
  }
});

const appWindow = getCurrentWindow();

document.getElementById('closeBtn').addEventListener('click', () => appWindow.close());
document.getElementById('minimizeBtn').addEventListener('click', () => appWindow.minimize());

const dragRegion = document.getElementById('dragRegion');
if (dragRegion) {
  dragRegion.addEventListener('mousedown', (e) => {
    if (e.target.closest('.close-btn, .minimize-btn')) return;
    appWindow.startDragging();
  });
}

// Submit initials
document.getElementById('submitBtn').addEventListener('click', async () => {
  const input = document.getElementById('initialsInput');
  const initials = input.value.toUpperCase().slice(0, 3).padEnd(3, ' ');
  const score = parseInt(document.getElementById('modalScore').textContent.replace(/,/g,'')) || 0;
  const tickets = parseInt(document.getElementById('modalTickets').textContent.replace(/,/g,'')) || 0;
  if (initials.trim()) updateScoreList(await invoke('submit_score', { initials, score, tickets }));
  document.getElementById('initialsModal').style.display = 'none';
  roundEndPending = false;
});
document.getElementById('initialsInput').addEventListener('keydown', (e) => {
  if (e.key === 'Enter') document.getElementById('submitBtn').click();
});
document.getElementById('initialsInput').addEventListener('input', (e) => {
  e.target.value = e.target.value.toUpperCase().replace(/[^A-Z]/g,'').slice(0,3);
});

// Change initials button
document.getElementById('changeInitialsBtn').addEventListener('click', () => {
  const input = document.getElementById('initialsInput');
  input.value = '';
  document.getElementById('initialsModal').style.display = 'flex';
  input.focus();
});

// Simulate
document.getElementById('simulateBtn').addEventListener('click', async () => {
  document.getElementById('simulateBtn').textContent = 'Simulating...';
  await invoke('simulate');
  roundEndPending = false;
  await updateDisplay();
  document.getElementById('simulateBtn').textContent = 'Sim Data';
});

document.getElementById('initialsModal').addEventListener('click', (e) => {
  if (e.target === e.currentTarget) {
    document.getElementById('initialsModal').style.display = 'none';
    roundEndPending = false;
  }
});

// Start
pollInterval = setInterval(updateDisplay, 200);
debugInterval = setInterval(updateDebugLog, 500);
updateDisplay();
