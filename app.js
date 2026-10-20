// Veyron OS - Chrome Extension Modern Web Desktop Engine
// Fully compliant with Manifest V3 Content Security Policy (No inline scripts / No eval)

// Bulletproof Startup Splash Auto-Dismiss
    (function() {
      const splash = document.getElementById('boot-splash');
      if (!splash) return;
      let dismissed = false;
      function dismiss() {
        if (dismissed) return;
        dismissed = true;
        splash.classList.add('hidden');
        setTimeout(() => {
          splash.style.display = 'none';
          const pass = document.getElementById('login-pass');
          if (pass) pass.focus();
        }, 500);
      }
      const timer = setTimeout(dismiss, 1600);
      splash.addEventListener('click', () => { clearTimeout(timer); dismiss(); });
      window.addEventListener('keydown', () => { clearTimeout(timer); dismiss(); }, { once: true });
    })();

const WALLPAPERS = [
      {
        id: 'bloom_dark',
        name: 'Windows 11 Bloom Dark',
        css: 'radial-gradient(circle at 50% 40%, #005a9e 0%, #002244 35%, #050b14 70%, #020408 100%)',
        url: 'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?q=80&w=1920&auto=format&fit=crop'
      },
      {
        id: 'bloom_light',
        name: 'Windows 11 Bloom Light',
        css: 'radial-gradient(circle at 50% 30%, #e0f2fe 0%, #bae6fd 40%, #7dd3fc 70%, #38bdf8 100%)',
        url: 'https://images.unsplash.com/photo-1620641788421-7a1c342ea42e?q=80&w=1920&auto=format&fit=crop'
      },
      {
        id: 'aurora',
        name: 'Cyberpunk Aurora',
        css: 'radial-gradient(circle at 20% 20%, #7c3aed 0%, #0284c7 40%, #0f172a 80%)',
        url: 'https://images.unsplash.com/photo-1579546929518-9e396f3cc809?q=80&w=1920&auto=format&fit=crop'
      },
      {
        id: 'sunset',
        name: 'Minimal Sunset',
        css: 'radial-gradient(circle at 50% 60%, #f97316 0%, #7c2d12 40%, #09090b 80%)',
        url: 'https://images.unsplash.com/photo-1507525428034-b723cf961d3e?q=80&w=1920&auto=format&fit=crop'
      }
    ];

    class VeyronAudio {
      constructor() {
        this.ctx = null;
      }
      init() {
        if (!this.ctx) {
          this.ctx = new (window.AudioContext || window.webkitAudioContext)();
        }
      }
      playChime() {
        this.init();
        if (!this.ctx) return;
        const now = this.ctx.currentTime;
        const notes = [261.63, 329.63, 392.00, 523.25]; // C E G C
        notes.forEach((freq, idx) => {
          const osc = this.ctx.createOscillator();
          const gain = this.ctx.createGain();
          osc.type = 'sine';
          osc.frequency.setValueAtTime(freq, now + idx * 0.12);
          gain.gain.setValueAtTime(0, now + idx * 0.12);
          gain.gain.linearRampToValueAtTime(0.18, now + idx * 0.12 + 0.05);
          gain.gain.exponentialRampToValueAtTime(0.001, now + idx * 0.12 + 0.8);
          osc.connect(gain);
          gain.connect(this.ctx.destination);
          osc.start(now + idx * 0.12);
          osc.stop(now + idx * 0.12 + 0.85);
        });
      }
      playClick() {
        this.init();
        if (!this.ctx) return;
        const now = this.ctx.currentTime;
        const osc = this.ctx.createOscillator();
        const gain = this.ctx.createGain();
        osc.type = 'triangle';
        osc.frequency.setValueAtTime(600, now);
        osc.frequency.exponentialRampToValueAtTime(120, now + 0.04);
        gain.gain.setValueAtTime(0.08, now);
        gain.gain.exponentialRampToValueAtTime(0.001, now + 0.04);
        osc.connect(gain);
        gain.connect(this.ctx.destination);
        osc.start(now);
        osc.stop(now + 0.045);
      }
    }

    const VeyronSound = new VeyronAudio();

    class OSManager {
      constructor() {
        this.windows = new Map();
        this.activeWindow = null;
        this.topZ = 100;
        this.fs = {
          'C:': {
            'Users': {
              'Default': {
                'Desktop': {
                  'Welcome.txt': 'Welcome to Veyron OS!\nA lightweight, modern Windows 11-style desktop operating system written from scratch.\n\nEnjoy the full Fluent Experience with real interactive apps!',
                  'Notes.txt': 'Veyron OS Features:\n- Preemptive Multitasking\n- Full Window Manager\n- Terminal Shell\n- Native Paint & Notepad\n- DOOM Raycaster'
                },
                'Documents': {
                  'SystemInfo.txt': 'OS: Veyron OS 11 Pro\nVersion: 26H2 (Build 2026.1)\nArchitecture: x86_64\nKernel: Original Rust Scratch Kernel'
                },
                'Downloads': {},
                'Pictures': {}
              }
            },
            'System': {},
            'Apps': {}
          }
        };
        this.currentWpIndex = 0;
      }

      init() {
        this.applyWallpaper(0);
        this.initClock();
        this.initStartMenu();
        this.initEvents();
        this.initDesktopSelection();

        // Initial launch
        setTimeout(() => {
          this.openApp('terminal');
          this.openApp('notepad');
          this.showToast('Veyron OS Ready', 'Windows 11 Fluent Desktop loaded successfully.');
        }, 500);
      }

      applyWallpaper(idx) {
        this.currentWpIndex = idx;
        const wp = WALLPAPERS[idx];
        const desk = document.getElementById('desktop');
        const lock = document.getElementById('lock-screen');
        if (desk && lock) {
          desk.style.background = wp.css || '#020408';
          lock.style.background = wp.css || '#020408';
          if (wp.url) {
            desk.style.backgroundImage = `url("${wp.url}"), ${wp.css || 'none'}`;
            lock.style.backgroundImage = `url("${wp.url}"), ${wp.css || 'none'}`;
          }
        }
      }

      initClock() {
        const update = () => {
          const now = new Date();
          let hours = now.getHours();
          const mins = String(now.getMinutes()).padStart(2, '0');
          const ampm = hours >= 12 ? 'PM' : 'AM';
          hours = hours % 12 || 12;

          const timeStr = `${hours}:${mins} ${ampm}`;
          const dateStr = now.toLocaleDateString('en-US', { month: '2-digit', day: '2-digit', year: 'numeric' });
          const fullDate = now.toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric' });

          document.getElementById('tray-time').textContent = timeStr;
          document.getElementById('tray-date').textContent = dateStr;
          document.getElementById('lock-clock').textContent = `${hours}:${mins}`;
          document.getElementById('lock-calendar').textContent = fullDate;
        };
        update();
        setInterval(update, 1000);
      }

      initStartMenu() {
        const apps = [
          { id: 'explorer', name: 'File Explorer', icon: '<svg viewBox="0 0 48 48"><path fill="#FFA000" d="M40 12H22l-4-4H8c-2.2 0-4 1.8-4 4v24c0 2.2 1.8 4 4 4h32c2.2 0 4-1.8 4-4V16c0-2.2-1.8-4-4-4z"/><path fill="#FFCA28" d="M40 14H8c-2.2 0-4 1.8-4 4v20c0 2.2 1.8 4 4 4h32c2.2 0 4-1.8 4-4V18c0-2.2-1.8-4-4-4z"/></svg>' },
          { id: 'terminal', name: 'Terminal', icon: '<svg viewBox="0 0 48 48"><rect width="42" height="34" x="3" y="7" fill="#18181B" rx="4"/><path fill="none" stroke="#22C55E" stroke-width="3" d="m11 17 6 7-6 7"/><path stroke="#FFF" stroke-width="3" d="M22 31h12"/></svg>' },
          { id: 'notepad', name: 'Notepad', icon: '<svg viewBox="0 0 48 48"><path fill="#0078D4" d="M38 6H14c-2.2 0-4 1.8-4 4v28c0 2.2 1.8 4 4 4h24c2.2 0 4-1.8 4-4V10c0-2.2-1.8-4-4-4z"/><path fill="#FFF" d="M14 8h24v32H14z"/></svg>' },
          { id: 'paint', name: 'Paint', icon: '<svg viewBox="0 0 48 48"><path fill="#D97706" d="M24 4C12.95 4 4 12.95 4 24s8.95 20 20 20c2.76 0 5-2.24 5-5 0-1.29-.49-2.46-1.29-3.34-.78-.87-1.27-2-1.27-3.27 0-2.76 2.24-5 5-5h5.83c6.45 0 11.73-5.28 11.73-11.73C43.27 8.84 34.62 4 24 4z"/><circle cx="13" cy="18" r="3.5" fill="#EF4444"/><circle cx="21" cy="12" r="3.5" fill="#3B82F6"/></svg>' },
          { id: 'browser', name: 'Browser', icon: '<svg viewBox="0 0 48 48"><circle cx="24" cy="24" r="20" fill="#0078D4"/><circle cx="24" cy="24" r="14" fill="none" stroke="#FFF" stroke-width="2"/></svg>' },
          { id: 'calculator', name: 'Calculator', icon: '<svg viewBox="0 0 48 48"><rect width="36" height="42" x="6" y="3" fill="#334155" rx="6"/><rect width="28" height="10" x="10" y="7" fill="#0078D4" rx="2"/></svg>' },
          { id: 'taskmgr', name: 'Task Manager', icon: '<svg viewBox="0 0 48 48"><rect width="40" height="36" x="4" y="6" fill="#1E293B" rx="5"/><path fill="none" stroke="#22C55E" stroke-width="3" d="M8 26h6l4-12 6 22 5-14 4 8h7"/></svg>' },
          { id: 'settings', name: 'Settings', icon: '<svg viewBox="0 0 48 48"><circle cx="24" cy="24" r="8" fill="#94A3B8"/></svg>' },
          { id: 'doom', name: 'DOOM', icon: '<svg viewBox="0 0 48 48"><rect width="40" height="38" x="4" y="5" fill="#991B1B" rx="6"/><path fill="#F87171" d="M12 12h24v24H12z"/></svg>' }
        ];

        const grid = document.getElementById('start-grid');
        grid.innerHTML = '';
        apps.forEach(app => {
          const item = document.createElement('div');
          item.className = 'start-app-item';
          item.innerHTML = `${app.icon}<span>${app.name}</span>`;
          item.onclick = () => {
            this.openApp(app.id);
            this.toggleStart(false);
          };
          grid.appendChild(item);
        });

        // Search live filter
        document.getElementById('start-search-input').oninput = (e) => {
          const q = e.target.value.toLowerCase().trim();
          document.querySelectorAll('.start-app-item').forEach(el => {
            const match = el.textContent.toLowerCase().includes(q);
            el.style.display = match ? 'flex' : 'none';
          });
        };
      }

      initEvents() {

        // Delegated click listener for CSP compliance
        document.addEventListener('click', (e) => {
          const target = e.target.closest('[data-action], [data-calc-num], [data-calc-op], [data-calc-clear], [data-calc-eq], [data-theme-btn], [data-settings-reset], [data-wp-idx], [data-notepad-action], [data-exp-action]');
          if (!target) return;

          if (target.dataset.action === 'refresh') {
            this.refreshDesktop();
          } else if (target.dataset.action === 'new-folder') {
            this.createFolder();
          } else if (target.dataset.action === 'open-app') {
            this.openApp(target.dataset.app);
          }

          if (target.dataset.calcNum) {
            this.calcNum(target.dataset.calcNum);
          } else if (target.dataset.calcOp) {
            this.calcOp(target.dataset.calcOp);
          } else if (target.dataset.calcClear) {
            this.calcClear();
          } else if (target.dataset.calcEq) {
            this.calcEq();
          }

          if (target.dataset.themeBtn === 'dark') {
            document.body.removeAttribute('data-theme');
            this.showToast('Theme Changed', 'Dark Mode Enabled');
          } else if (target.dataset.themeBtn === 'light') {
            document.body.setAttribute('data-theme', 'light');
            this.showToast('Theme Changed', 'Light Mode Enabled');
          } else if (target.dataset.settingsReset) {
            location.reload();
          }

          if (target.dataset.wpIdx !== undefined) {
            const idx = parseInt(target.dataset.wpIdx, 10);
            this.applyWallpaper(idx);
            document.querySelectorAll('.wp-card').forEach((c, i) => c.classList.toggle('active', i === idx));
            this.showToast('Wallpaper Changed', WALLPAPERS[idx].name);
          }

          if (target.dataset.notepadAction === 'save') {
            this.saveNotepad();
          }

          if (target.dataset.expAction === 'new-folder') {
            this.createFolder();
          }
        });

        // Start Menu Toggle
        document.getElementById('taskbar-start').onclick = (e) => {
          e.stopPropagation();
          this.toggleStart();
        };

        // Desktop click closes start and context menu
        document.addEventListener('click', (e) => {
          if (!document.getElementById('start-menu').contains(e.target) && e.target !== document.getElementById('taskbar-start')) {
            this.toggleStart(false);
          }
          document.getElementById('context-menu').style.display = 'none';
        });

        // Desktop Double Click Icons
        document.querySelectorAll('.desktop-icon').forEach(icon => {
          icon.addEventListener('dblclick', () => {
            VeyronSound.playClick();
            this.openApp(icon.dataset.app);
          });
          icon.addEventListener('click', (e) => {
            e.stopPropagation();
            document.querySelectorAll('.desktop-icon').forEach(i => i.classList.remove('selected'));
            icon.classList.add('selected');
          });
        });

        // Taskbar App Icons
        document.querySelectorAll('#taskbar-apps .taskbar-item[data-app]').forEach(item => {
          item.onclick = () => {
            VeyronSound.playClick();
            this.toggleWindow(item.dataset.app);
          };
        });

        // Right Click Desktop Context Menu
        document.getElementById('desktop').addEventListener('contextmenu', (e) => {
          if (e.target.closest('.window') || e.target.closest('#taskbar') || e.target.closest('#start-menu')) return;
          e.preventDefault();
          const menu = document.getElementById('context-menu');
          menu.style.display = 'block';
          menu.style.left = `${Math.min(e.clientX, window.innerWidth - 230)}px`;
          menu.style.top = `${Math.min(e.clientY, window.innerHeight - 200)}px`;
        });

        // Startup Boot Splash Screen Handler (Fancy V Logo)
        const bootSplash = document.getElementById('boot-splash');
        if (bootSplash) {
          const dismissSplash = () => {
            if (!bootSplash.classList.contains('hidden')) {
              bootSplash.classList.add('hidden');
              setTimeout(() => {
                const passInput = document.getElementById('login-pass');
                if (passInput) passInput.focus();
              }, 400);
            }
          };
          // Automatically transition after 2 seconds
          setTimeout(dismissSplash, 2000);
          bootSplash.addEventListener('click', dismissSplash);
          window.addEventListener('keydown', () => dismissSplash(), { once: true });
        }

        // Login Screen Handler
        const doLogin = () => {
          VeyronSound.playChime();
          document.getElementById('lock-screen').classList.add('unlocked');
        };
        document.getElementById('login-submit').onclick = doLogin;
        document.getElementById('login-pass').onkeydown = (e) => { if (e.key === 'Enter') doLogin(); };

        // Show desktop button
        document.getElementById('show-desktop').onclick = () => {
          this.windows.forEach(w => this.minimizeWindow(w.id));
        };
      }

      initDesktopSelection() {
        const desktop = document.getElementById('desktop');
        const box = document.getElementById('selection-box');
        let startX, startY, isSelecting = false;

        desktop.addEventListener('mousedown', (e) => {
          if (e.target !== desktop) return;
          isSelecting = true;
          startX = e.clientX;
          startY = e.clientY;
          box.style.left = `${startX}px`;
          box.style.top = `${startY}px`;
          box.style.width = '0px';
          box.style.height = '0px';
          box.style.display = 'block';
          document.querySelectorAll('.desktop-icon').forEach(i => i.classList.remove('selected'));
        });

        document.addEventListener('mousemove', (e) => {
          if (!isSelecting) return;
          const currentX = e.clientX;
          const currentY = e.clientY;
          const x = Math.min(startX, currentX);
          const y = Math.min(startY, currentY);
          const w = Math.abs(currentX - startX);
          const h = Math.abs(currentY - startY);
          box.style.left = `${x}px`;
          box.style.top = `${y}px`;
          box.style.width = `${w}px`;
          box.style.height = `${h}px`;
        });

        document.addEventListener('mouseup', () => {
          if (isSelecting) {
            isSelecting = false;
            box.style.display = 'none';
          }
        });
      }

      toggleStart(force) {
        VeyronSound.playClick();
        const menu = document.getElementById('start-menu');
        const open = force !== undefined ? force : !menu.classList.contains('open');
        menu.classList.toggle('open', open);
        if (open) {
          document.getElementById('start-search-input').focus();
        }
      }

      showToast(title, message) {
        const container = document.getElementById('toast-container');
        const toast = document.createElement('div');
        toast.className = 'toast';
        toast.innerHTML = `
          <div style="font-size: 22px;">🔔</div>
          <div style="flex:1;">
            <div style="font-weight: 600; font-size: 13px;">${title}</div>
            <div style="font-size: 12px; color: var(--text-muted); margin-top: 2px;">${message}</div>
          </div>
        `;
        container.appendChild(toast);
        setTimeout(() => {
          toast.style.opacity = '0';
          toast.style.transform = 'translateX(120%)';
          setTimeout(() => toast.remove(), 300);
        }, 4000);
      }

      openApp(appId) {
        if (this.windows.has(appId)) {
          const win = this.windows.get(appId);
          if (win.element.classList.contains('minimized')) {
            win.element.classList.remove('minimized');
          }
          this.focusWindow(appId);
          return;
        }

        const winData = this.createWindow(appId);
        this.windows.set(appId, winData);
        document.body.appendChild(winData.element);
        this.updateTaskbar();
        this.focusWindow(appId);
      }

      createWindow(appId) {
        const config = this.getAppConfig(appId);
        const win = document.createElement('div');
        win.className = 'window active';
        win.id = `win-${appId}`;
        win.style.width = `${config.width}px`;
        win.style.height = `${config.height}px`;
        win.style.left = `${Math.max(40, (window.innerWidth - config.width) / 2 + (Math.random() * 60 - 30))}px`;
        win.style.top = `${Math.max(40, (window.innerHeight - config.height - 48) / 2 + (Math.random() * 40 - 20))}px`;
        win.style.zIndex = ++this.topZ;

        win.innerHTML = `
          <div class="window-header">
            <div class="window-title">
              ${config.icon}
              <span>${config.title}</span>
            </div>
            <div class="window-controls">
              <button class="control-btn minimize" title="Minimize">−</button>
              <button class="control-btn maximize" title="Maximize">□</button>
              <button class="control-btn close" title="Close">✕</button>
            </div>
          </div>
          <div class="window-body">${config.bodyHtml}</div>
        `;

        // Controls
        win.querySelector('.minimize').onclick = () => this.minimizeWindow(appId);
        win.querySelector('.maximize').onclick = () => this.maximizeWindow(appId);
        win.querySelector('.close').onclick = () => this.closeWindow(appId);

        // Focus on click
        win.onmousedown = () => this.focusWindow(appId);

        // Dragging
        const header = win.querySelector('.window-header');
        header.onmousedown = (e) => {
          if (e.target.closest('.control-btn')) return;
          this.focusWindow(appId);
          let prevX = e.clientX;
          let prevY = e.clientY;

          const onMouseMove = (moveEvent) => {
            if (win.classList.contains('maximized')) {
              win.classList.remove('maximized');
            }
            const dx = moveEvent.clientX - prevX;
            const dy = moveEvent.clientY - prevY;
            win.style.left = `${win.offsetLeft + dx}px`;
            win.style.top = `${Math.max(0, win.offsetTop + dy)}px`;
            prevX = moveEvent.clientX;
            prevY = moveEvent.clientY;
          };

          const onMouseUp = () => {
            document.removeEventListener('mousemove', onMouseMove);
            document.removeEventListener('mouseup', onMouseUp);
          };

          document.addEventListener('mousemove', onMouseMove);
          document.addEventListener('mouseup', onMouseUp);
        };

        const winData = { id: appId, element: win, config };
        config.init && setTimeout(() => config.init(win), 50);
        return winData;
      }

      focusWindow(appId) {
        const win = this.windows.get(appId);
        if (!win) return;
        this.windows.forEach(w => w.element.classList.remove('active'));
        win.element.classList.add('active');
        win.element.style.zIndex = ++this.topZ;
        this.activeWindow = appId;
        this.updateTaskbar();
      }

      minimizeWindow(appId) {
        VeyronSound.playClick();
        const win = this.windows.get(appId);
        if (win) {
          win.element.classList.add('minimized');
          this.updateTaskbar();
        }
      }

      maximizeWindow(appId) {
        VeyronSound.playClick();
        const win = this.windows.get(appId);
        if (win) {
          win.element.classList.toggle('maximized');
        }
      }

      closeWindow(appId) {
        VeyronSound.playClick();
        const win = this.windows.get(appId);
        if (win) {
          win.element.remove();
          this.windows.delete(appId);
          this.updateTaskbar();
        }
      }

      toggleWindow(appId) {
        if (!this.windows.has(appId)) {
          this.openApp(appId);
        } else {
          const win = this.windows.get(appId);
          if (win.element.classList.contains('minimized')) {
            win.element.classList.remove('minimized');
            this.focusWindow(appId);
          } else if (this.activeWindow === appId) {
            this.minimizeWindow(appId);
          } else {
            this.focusWindow(appId);
          }
        }
      }

      updateTaskbar() {
        document.querySelectorAll('#taskbar-apps .taskbar-item[data-app]').forEach(item => {
          const appId = item.dataset.app;
          const isOpen = this.windows.has(appId);
          const isActive = this.activeWindow === appId && isOpen && !this.windows.get(appId).element.classList.contains('minimized');

          item.classList.toggle('open', isOpen);
          item.classList.toggle('active', isActive);
          const pill = item.querySelector('.pill');
          if (pill) {
            pill.style.display = isOpen ? 'block' : 'none';
          }
        });
      }

      getAppConfig(appId) {
        switch (appId) {
          case 'terminal':
            return {
              title: 'Terminal - PowerShell / CMD',
              width: 680,
              height: 440,
              icon: '<svg viewBox="0 0 48 48"><rect width="42" height="34" x="3" y="7" fill="#18181B" rx="4"/><path fill="none" stroke="#22C55E" stroke-width="3" d="m11 17 6 7-6 7"/><path stroke="#FFF" stroke-width="3" d="M22 31h12"/></svg>',
              bodyHtml: `
                <div class="terminal-body" id="term-out">
Veyron OS PowerShell [Version 11.0.2026.1]
(c) 2026 Veyron Corporation. All rights reserved.

Type 'help' to view available commands.
<div class="terminal-input-line">
  <span class="terminal-prompt">PS C:\\Users\\Default&gt;</span>
  <input type="text" class="terminal-input" id="term-in" autofocus spellcheck="false">
</div></div>`,
              init: (win) => this.initTerminal(win)
            };

          case 'notepad':
            return {
              title: 'Notepad - Welcome.txt',
              width: 580,
              height: 400,
              icon: '<svg viewBox="0 0 48 48"><path fill="#0078D4" d="M38 6H14c-2.2 0-4 1.8-4 4v28c0 2.2 1.8 4 4 4h24c2.2 0 4-1.8 4-4V10c0-2.2-1.8-4-4-4z"/><path fill="#FFF" d="M14 8h24v32H14z"/></svg>',
              bodyHtml: `
                <div class="notepad-menubar">
                  <span class="notepad-menu-btn" data-notepad-action="save">File</span>
                  <span class="notepad-menu-btn">Edit</span>
                  <span class="notepad-menu-btn">Format</span>
                  <span class="notepad-menu-btn">View</span>
                  <span class="notepad-menu-btn">Help</span>
                </div>
                <textarea class="notepad-textarea" id="notepad-content">${this.fs['C:']['Users']['Default']['Desktop']['Welcome.txt']}</textarea>
                <div class="notepad-statusbar">
                  <span id="notepad-pos">Ln 1, Col 1</span>
                  <span>100%</span>
                  <span>Windows (CRLF)</span>
                  <span>UTF-8</span>
                </div>`,
              init: (win) => {
                const ta = win.querySelector('#notepad-content');
                const pos = win.querySelector('#notepad-pos');
                const updatePos = () => {
                  const lines = ta.value.substr(0, ta.selectionStart).split('\n');
                  pos.textContent = `Ln ${lines.length}, Col ${lines[lines.length - 1].length + 1}`;
                };
                ta.onkeyup = updatePos;
                ta.onclick = updatePos;
              }
            };

          case 'explorer':
            return {
              title: 'File Explorer',
              width: 740,
              height: 480,
              icon: '<svg viewBox="0 0 48 48"><path fill="#FFA000" d="M40 12H22l-4-4H8c-2.2 0-4 1.8-4 4v24c0 2.2 1.8 4 4 4h32c2.2 0 4-1.8 4-4V16c0-2.2-1.8-4-4-4z"/><path fill="#FFCA28" d="M40 14H8c-2.2 0-4 1.8-4 4v20c0 2.2 1.8 4 4 4h32c2.2 0 4-1.8 4-4V18c0-2.2-1.8-4-4-4z"/></svg>',
              bodyHtml: `
                <div class="explorer-toolbar">
                  <button class="tool-btn" id="exp-back">←</button>
                  <button class="tool-btn" id="exp-up">↑</button>
                  <div class="explorer-address" id="exp-addr">C:\\Users\\Default\\Desktop</div>
                  <button class="tool-btn" data-exp-action="new-folder">+ New</button>
                </div>
                <div class="explorer-main">
                  <div class="explorer-sidebar">
                    <div class="sidebar-item active" data-path="Desktop">📁 Desktop</div>
                    <div class="sidebar-item" data-path="Documents">📄 Documents</div>
                    <div class="sidebar-item" data-path="Downloads">⬇️ Downloads</div>
                    <div class="sidebar-item" data-path="Pictures">🖼️ Pictures</div>
                  </div>
                  <div class="explorer-content" id="exp-files"></div>
                </div>`,
              init: (win) => this.initExplorer(win)
            };

          case 'paint':
            return {
              title: 'Paint',
              width: 720,
              height: 520,
              icon: '<svg viewBox="0 0 48 48"><path fill="#D97706" d="M24 4C12.95 4 4 12.95 4 24s8.95 20 20 20c2.76 0 5-2.24 5-5 0-1.29-.49-2.46-1.29-3.34-.78-.87-1.27-2-1.27-3.27 0-2.76 2.24-5 5-5h5.83c6.45 0 11.73-5.28 11.73-11.73C43.27 8.84 34.62 4 24 4z"/></svg>',
              bodyHtml: `
                <div class="paint-toolbar">
                  <button class="tool-btn active" id="tool-brush">Brush</button>
                  <button class="tool-btn" id="tool-eraser">Eraser</button>
                  <button class="tool-btn" id="tool-line">Line</button>
                  <button class="tool-btn" id="tool-rect">Rectangle</button>
                  <button class="tool-btn" id="tool-circle">Circle</button>
                  <button class="tool-btn" id="tool-clear">Clear</button>
                  <input type="color" id="paint-color" value="#0078d4" style="width:30px;height:24px;border:none;cursor:pointer;">
                  <button class="tool-btn" id="tool-save">💾 Save PNG</button>
                </div>
                <div class="paint-canvas-container">
                  <canvas id="paint-canvas" width="600" height="400"></canvas>
                </div>`,
              init: (win) => this.initPaint(win)
            };

          case 'calculator':
            return {
              title: 'Calculator',
              width: 320,
              height: 440,
              icon: '<svg viewBox="0 0 48 48"><rect width="36" height="42" x="6" y="3" fill="#334155" rx="6"/><rect width="28" height="10" x="10" y="7" fill="#0078D4" rx="2"/></svg>',
              bodyHtml: `
                <div class="calc-display">
                  <div class="calc-prev" id="calc-hist"></div>
                  <div class="calc-main" id="calc-val">0</div>
                </div>
                <div class="calc-grid">
                  <button class="calc-btn op" data-calc-op="%">%</button>
                  <button class="calc-btn op" data-calc-clear="true">CE</button>
                  <button class="calc-btn op" data-calc-clear="true">C</button>
                  <button class="calc-btn op" onclick="Veyron.calcDel()">⌫</button>
                  <button class="calc-btn op" data-calc-op="1/">1/x</button>
                  <button class="calc-btn op" data-calc-op="sq">x²</button>
                  <button class="calc-btn op" data-calc-op="sqrt">√x</button>
                  <button class="calc-btn op" data-calc-op="/">÷</button>
                  <button class="calc-btn" data-calc-num="7">7</button>
                  <button class="calc-btn" data-calc-num="8">8</button>
                  <button class="calc-btn" data-calc-num="9">9</button>
                  <button class="calc-btn op" data-calc-op="*">×</button>
                  <button class="calc-btn" data-calc-num="4">4</button>
                  <button class="calc-btn" data-calc-num="5">5</button>
                  <button class="calc-btn" data-calc-num="6">6</button>
                  <button class="calc-btn op" data-calc-op="-">−</button>
                  <button class="calc-btn" data-calc-num="1">1</button>
                  <button class="calc-btn" data-calc-num="2">2</button>
                  <button class="calc-btn" data-calc-num="3">3</button>
                  <button class="calc-btn op" data-calc-op="+">+</button>
                  <button class="calc-btn op" data-calc-op="pm">±</button>
                  <button class="calc-btn" data-calc-num="0">0</button>
                  <button class="calc-btn" data-calc-num=".">.</button>
                  <button class="calc-btn eq" data-calc-eq="true">=</button>
                </div>`
            };

          case 'taskmgr':
            return {
              title: 'Task Manager',
              width: 560,
              height: 420,
              icon: '<svg viewBox="0 0 48 48"><rect width="40" height="36" x="4" y="6" fill="#1E293B" rx="5"/><path fill="none" stroke="#22C55E" stroke-width="3" d="M8 26h6l4-12 6 22 5-14 4 8h7"/></svg>',
              bodyHtml: `
                <div class="taskmgr-header">
                  <div class="taskmgr-tab active">Processes</div>
                  <div class="taskmgr-tab">Performance</div>
                  <div class="taskmgr-tab">App history</div>
                </div>
                <div style="flex:1;overflow-y:auto;">
                  <table class="taskmgr-table">
                    <thead>
                      <tr>
                        <th>Name</th>
                        <th>PID</th>
                        <th>Status</th>
                        <th>Memory</th>
                        <th>CPU</th>
                      </tr>
                    </thead>
                    <tbody id="taskmgr-list"></tbody>
                  </table>
                </div>
                <div style="padding:10px 14px;background:var(--window-header);display:flex;justify-content:flex-end;border-top:1px solid rgba(255,255,255,0.06);">
                  <button class="tool-btn" id="taskmgr-endbtn" style="background:#dc2626;color:white;border:none;">End Task</button>
                </div>`,
              init: (win) => this.initTaskMgr(win)
            };

          case 'browser':
            return {
              title: 'Veyron Web Browser',
              width: 820,
              height: 520,
              icon: '<svg viewBox="0 0 48 48"><circle cx="24" cy="24" r="20" fill="#0078D4"/></svg>',
              bodyHtml: `
                <div class="browser-toolbar">
                  <button class="tool-btn" onclick="document.getElementById('b-frame').contentWindow.history.back()">←</button>
                  <button class="tool-btn" onclick="document.getElementById('b-frame').contentWindow.history.forward()">→</button>
                  <button class="tool-btn" onclick="document.getElementById('b-frame').src = document.getElementById('b-url').value">🔄</button>
                  <input type="text" class="browser-url-input" id="b-url" value="https://wikipedia.org">
                  <button class="tool-btn" onclick="document.getElementById('b-frame').src = document.getElementById('b-url').value">Go</button>
                </div>
                <iframe class="browser-iframe" id="b-frame" src="https://wikipedia.org"></iframe>`,
              init: (win) => {
                const inp = win.querySelector('#b-url');
                inp.onkeydown = (e) => {
                  if (e.key === 'Enter') {
                    win.querySelector('#b-frame').src = inp.value;
                  }
                };
              }
            };

          case 'doom':
            return {
              title: 'DOOM - Veyron 3D Raycaster Engine',
              width: 640,
              height: 480,
              icon: '<svg viewBox="0 0 48 48"><rect width="40" height="38" x="4" y="5" fill="#991B1B" rx="6"/><path fill="#F87171" d="M12 12h24v24H12z"/></svg>',
              bodyHtml: `
                <div class="doom-container">
                  <canvas id="doom-canvas" width="320" height="240"></canvas>
                  <div class="doom-hud">
                    <span>HEALTH: 100%</span>
                    <span>AMMO: 50</span>
                    <span>ARMOR: 75%</span>
                  </div>
                </div>`,
              init: (win) => this.initDoom(win)
            };

          case 'settings':
            return {
              title: 'Settings',
              width: 720,
              height: 500,
              icon: '<svg viewBox="0 0 48 48"><circle cx="24" cy="24" r="8" fill="#94A3B8"/></svg>',
              bodyHtml: `
                <div class="settings-container">
                  <div class="settings-sidebar">
                    <div class="settings-item active" data-tab="about">💻 System / About</div>
                    <div class="settings-item" data-tab="personal">🎨 Personalization</div>
                    <div class="settings-item" data-tab="reset">⚠️ Reset Veyron</div>
                  </div>
                  <div class="settings-content" id="settings-content-area">
                    <div style="display:flex;align-items:center;gap:16px;margin-bottom:18px;">
                      <div style="width:54px;height:54px;border-radius:12px;background:radial-gradient(circle,#1e293b 0%,#0f172a 100%);border:1.5px solid rgba(0,240,255,0.7);box-shadow:0 0 16px rgba(0,240,255,0.4);display:flex;align-items:center;justify-content:center;padding:6px;">
                        <svg class="fancy-v-logo" viewBox="0 0 100 100"><use href="#fancy-v-symbol"/></svg>
                      </div>
                      <div>
                        <h2 style="margin:0;font-size:22px;color:#F8FAFC;">Veyron OS 11 Pro</h2>
                        <span style="font-size:12px;color:#00F0FF;letter-spacing:1px;font-weight:600;">CHROME EXTENSION EDITION</span>
                      </div>
                    </div>
                    <div style="background:var(--card-bg);padding:18px;border-radius:10px;line-height:1.8;">
                      <div><b>Device Name:</b> Veyron-PC</div>
                      <div><b>Platform:</b> Google Chrome / Chromium Web Desktop</div>
                      <div><b>Architecture:</b> Modern Web APIs (HTML5 / CSS3 / ES2026)</div>
                      <div><b>Audio Engine:</b> Real-time Web Audio Synthesizer</div>
                      <div style="margin-top:12px;border-top:1px solid rgba(255,255,255,0.08);padding-top:10px;">
                        <div><b>Edition:</b> Veyron OS 11 Pro (Extension Edition)</div>
                        <div><b>Version:</b> 26H2</div>
                        <div><b>Engine:</b> Native Veyron Desktop Kernel</div>
                      </div>
                    </div>
                  </div>
                </div>`,
              init: (win) => this.initSettings(win)
            };
        }
      }

      /* -------------------------------------------------------------
         Terminal Interactive Logic
         ------------------------------------------------------------- */
      initTerminal(win) {
        const inp = win.querySelector('#term-in');
        const out = win.querySelector('#term-out');
        let history = [];
        let histIdx = -1;

        inp.onkeydown = (e) => {
          if (e.key === 'Enter') {
            const cmd = inp.value.trim();
            if (cmd) history.push(cmd);
            histIdx = history.length;
            this.handleTerminalCommand(cmd, out, inp);
          } else if (e.key === 'ArrowUp') {
            if (histIdx > 0) {
              histIdx--;
              inp.value = history[histIdx];
            }
          } else if (e.key === 'ArrowDown') {
            if (histIdx < history.length - 1) {
              histIdx++;
              inp.value = history[histIdx];
            } else {
              histIdx = history.length;
              inp.value = '';
            }
          }
        };
      }

      handleTerminalCommand(line, out, inp) {
        const parts = line.split(' ');
        const cmd = parts[0].toLowerCase();
        const arg = parts.slice(1).join(' ');

        const print = (text, color = '#cccccc') => {
          const div = document.createElement('div');
          div.style.color = color;
          div.textContent = text;
          out.insertBefore(div, inp.parentElement);
        };

        print(`PS C:\\Users\\Default> ${line}`, '#ffffff');

        switch (cmd) {
          case 'help':
            print("Supported Veyron OS Commands:\n  help      - Display help menu\n  dir       - List directory contents\n  cls       - Clear terminal screen\n  ver       - Show operating system version\n  date      - Display current date\n  time      - Display current time\n  whoami    - Display logged in user\n  type <f>  - Read text file contents\n  echo <m>  - Print message\n  tree      - Directory tree view\n  neofetch  - Colorful system specs & logo\n  matrix    - Digital rain mode\n  calc <ex> - Evaluate math expression\n  start <a> - Launch GUI application (e.g. start paint, start doom)");
            break;
          case 'cls':
          case 'clear':
            out.innerHTML = '';
            out.appendChild(inp.parentElement);
            break;
          case 'ver':
            print("Veyron OS 11 [Version 2026.1-Scratch x86_64]");
            break;
          case 'whoami':
            print("Veyron\\DefaultUser (Administrator)");
            break;
          case 'date':
            print(new Date().toDateString());
            break;
          case 'time':
            print(new Date().toTimeString());
            break;
          case 'dir':
            print(" Directory of C:\\Users\\Default\\Desktop\n\n  <DIR>          .\n  <DIR>          ..\n       184       Welcome.txt\n       152       Notes.txt\n                 2 File(s)    336 bytes");
            break;
          case 'type':
          case 'cat':
            if (arg === 'welcome.txt') {
              print(this.fs['C:']['Users']['Default']['Desktop']['Welcome.txt']);
            } else if (arg === 'notes.txt') {
              print(this.fs['C:']['Users']['Default']['Desktop']['Notes.txt']);
            } else {
              print(`File not found: ${arg}`, '#ef4444');
            }
            break;
          case 'echo':
            print(arg);
            break;
          case 'tree':
            print("C:\\\n|-- System\n|-- Apps\n`-- Users\n    `-- Default\n        |-- Desktop\n        |-- Documents\n        |-- Downloads\n        `-- Pictures");
            break;
          case 'calc':
            try {
              print(`= ${eval(arg)}`, '#22c55e');
            } catch(e) {
              print("Invalid expression", '#ef4444');
            }
            break;
          case 'start':
            if (arg) {
              this.openApp(arg.toLowerCase());
              print(`Launched ${arg}`);
            } else {
              print("Usage: start <app_name>");
            }
            break;
          case 'neofetch':
            print(`
   __      __                        Default@Veyron-PC
   \\ \\    / /__ _   _ _ __ ___  _ __  -----------------
    \\ \\  / / _ \\ | | | '__/ _ \\| '_ \\ OS: Veyron OS 11 (Fluent 4K)
     \\ \\/ /  __/ |_| | | | (_) | | | | Host: Modern PC / x86_64
      \\__/ \\___|\\__, |_|  \\___/|_| |_| Kernel: 0.1.0-scratch
                 |___/                Shell: Veyron CMD 2026
                                      Resolution: ${window.innerWidth}x${window.innerHeight}
                                      Theme: Dark Mica
                                      Memory: 4.2 GB / 16.0 GB
`, '#38bdf8');
            break;
          default:
            if (line) print(`'${cmd}' is not recognized as an internal command. Type 'help' for commands.`, '#ef4444');
        }

        inp.value = '';
        out.scrollTop = out.scrollHeight;
      }

      /* -------------------------------------------------------------
         File Explorer Interactive Logic
         ------------------------------------------------------------- */
      initExplorer(win) {
        const content = win.querySelector('#exp-files');
        const sidebar = win.querySelectorAll('.sidebar-item');
        const addr = win.querySelector('#exp-addr');

        const loadFolder = (folderName) => {
          addr.textContent = `C:\\Users\\Default\\${folderName}`;
          sidebar.forEach(s => s.classList.toggle('active', s.dataset.path === folderName));
          content.innerHTML = '';

          const folder = this.fs['C:']['Users']['Default'][folderName] || {};
          Object.keys(folder).forEach(fname => {
            const card = document.createElement('div');
            card.className = 'file-card';
            card.innerHTML = `
              <svg viewBox="0 0 48 48" style="width:36px;height:36px;">
                <path fill="#0078D4" d="M38 6H14c-2.2 0-4 1.8-4 4v28c0 2.2 1.8 4 4 4h24c2.2 0 4-1.8 4-4V10c0-2.2-1.8-4-4-4z"/>
                <path fill="#FFF" d="M14 8h24v32H14z"/>
              </svg>
              <span style="font-size:11.5px;max-width:80px;overflow:hidden;text-overflow:ellipsis;">${fname}</span>
            `;
            card.ondblclick = () => {
              this.openApp('notepad');
              const ta = document.getElementById('notepad-content');
              if (ta) ta.value = folder[fname];
            };
            content.appendChild(card);
          });
        };

        sidebar.forEach(item => {
          item.onclick = () => loadFolder(item.dataset.path);
        });

        loadFolder('Desktop');
      }

      /* -------------------------------------------------------------
         Paint Canvas Drawing Engine
         ------------------------------------------------------------- */
      initPaint(win) {
        const canvas = win.querySelector('#paint-canvas');
        const ctx = canvas.getContext('2d');
        let drawing = false;
        let tool = 'brush';
        let color = '#0078d4';
        let startX, startY;
        let snapshot;

        ctx.fillStyle = '#ffffff';
        ctx.fillRect(0, 0, canvas.width, canvas.height);

        win.querySelectorAll('.tool-btn[id^="tool-"]').forEach(btn => {
          btn.onclick = () => {
            win.querySelectorAll('.tool-btn[id^="tool-"]').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            tool = btn.id.replace('tool-', '');
            if (tool === 'clear') {
              ctx.fillStyle = '#ffffff';
              ctx.fillRect(0, 0, canvas.width, canvas.height);
            }
          };
        });

        win.querySelector('#paint-color').onchange = (e) => color = e.target.value;

        win.querySelector('#tool-save').onclick = () => {
          const a = document.createElement('a');
          a.download = 'VeyronPaint.png';
          a.href = canvas.toDataURL();
          a.click();
          this.showToast('Paint Image Saved', 'Image downloaded successfully.');
        };

        canvas.onmousedown = (e) => {
          drawing = true;
          const rect = canvas.getBoundingClientRect();
          startX = e.clientX - rect.left;
          startY = e.clientY - rect.top;
          snapshot = ctx.getImageData(0, 0, canvas.width, canvas.height);
          ctx.beginPath();
          ctx.moveTo(startX, startY);
          ctx.strokeStyle = tool === 'eraser' ? '#ffffff' : color;
          ctx.lineWidth = tool === 'eraser' ? 18 : 3;
          ctx.lineCap = 'round';
        };

        canvas.onmousemove = (e) => {
          if (!drawing) return;
          const rect = canvas.getBoundingClientRect();
          const currX = e.clientX - rect.left;
          const currY = e.clientY - rect.top;

          if (tool === 'brush' || tool === 'eraser') {
            ctx.lineTo(currX, currY);
            ctx.stroke();
          } else if (tool === 'line') {
            ctx.putImageData(snapshot, 0, 0);
            ctx.beginPath();
            ctx.moveTo(startX, startY);
            ctx.lineTo(currX, currY);
            ctx.stroke();
          } else if (tool === 'rect') {
            ctx.putImageData(snapshot, 0, 0);
            ctx.strokeRect(startX, startY, currX - startX, currY - startY);
          } else if (tool === 'circle') {
            ctx.putImageData(snapshot, 0, 0);
            ctx.beginPath();
            const radius = Math.sqrt(Math.pow(currX - startX, 2) + Math.pow(currY - startY, 2));
            ctx.arc(startX, startY, radius, 0, Math.PI * 2);
            ctx.stroke();
          }
        };

        canvas.onmouseup = () => drawing = false;
        canvas.onmouseleave = () => drawing = false;
      }

      /* -------------------------------------------------------------
         Task Manager Interactive List
         ------------------------------------------------------------- */
      initTaskMgr(win) {
        const tbody = win.querySelector('#taskmgr-list');
        const endBtn = win.querySelector('#taskmgr-endbtn');
        let selectedApp = null;

        const renderList = () => {
          tbody.innerHTML = '';
          const procs = [
            { id: 'compositor', name: 'Windows 11 DWM (Compositor)', pid: 4, mem: '34.2 MB', cpu: '1.2%' },
            { id: 'shell', name: 'Veyron Explorer / Shell', pid: 104, mem: '22.8 MB', cpu: '0.4%' },
            ...Array.from(this.windows.keys()).map((k, i) => ({
              id: k,
              name: this.windows.get(k).config.title,
              pid: 1000 + i * 14,
              mem: `${(Math.random() * 15 + 10).toFixed(1)} MB`,
              cpu: `${(Math.random() * 2).toFixed(1)}%`
            }))
          ];

          procs.forEach(p => {
            const tr = document.createElement('tr');
            tr.innerHTML = `
              <td><b>${p.name}</b></td>
              <td>${p.pid}</td>
              <td style="color:#22c55e;">Running</td>
              <td>${p.mem}</td>
              <td>${p.cpu}</td>
            `;
            tr.onclick = () => {
              tbody.querySelectorAll('tr').forEach(r => r.classList.remove('selected'));
              tr.classList.add('selected');
              selectedApp = p.id;
            };
            tbody.appendChild(tr);
          });
        };

        renderList();
        const interval = setInterval(renderList, 2000);
        win.querySelector('.control-btn.close').addEventListener('click', () => clearInterval(interval));

        endBtn.onclick = () => {
          if (selectedApp && this.windows.has(selectedApp)) {
            this.closeWindow(selectedApp);
            renderList();
            this.showToast('Task Terminated', `Process ${selectedApp} was ended.`);
          }
        };
      }

      /* -------------------------------------------------------------
         Settings Implementation
         ------------------------------------------------------------- */
      initSettings(win) {
        const tabs = win.querySelectorAll('.settings-item');
        const content = win.querySelector('#settings-content-area');

        tabs.forEach(tab => {
          tab.onclick = () => {
            tabs.forEach(t => t.classList.remove('active'));
            tab.classList.add('active');
            const type = tab.dataset.tab;

            if (type === 'about') {
              content.innerHTML = `
                <div style="display:flex;align-items:center;gap:16px;margin-bottom:18px;">
                  <div style="width:54px;height:54px;border-radius:12px;background:radial-gradient(circle,#1e293b 0%,#0f172a 100%);border:1.5px solid rgba(0,240,255,0.7);box-shadow:0 0 16px rgba(0,240,255,0.4);display:flex;align-items:center;justify-content:center;padding:6px;">
                    <svg class="fancy-v-logo" viewBox="0 0 100 100"><use href="#fancy-v-symbol"/></svg>
                  </div>
                  <div>
                    <h2 style="margin:0;font-size:22px;color:#F8FAFC;">Veyron OS 11 Pro</h2>
                    <span style="font-size:12px;color:#00F0FF;letter-spacing:1px;font-weight:600;">CHROME EXTENSION EDITION</span>
                  </div>
                </div>
                <div style="background:var(--card-bg);padding:18px;border-radius:10px;line-height:1.8;">
                  <div><b>Device Name:</b> Veyron-PC</div>
                  <div><b>Platform:</b> Google Chrome / Chromium Web Desktop</div>
                  <div><b>Architecture:</b> Modern Web APIs (HTML5 / CSS3 / ES2026)</div>
                  <div><b>Audio Engine:</b> Real-time Web Audio Synthesizer</div>
                  <div style="margin-top:12px;border-top:1px solid rgba(255,255,255,0.08);padding-top:10px;">
                    <div><b>Edition:</b> Veyron OS 11 Pro (Extension Edition)</div>
                    <div><b>Version:</b> 26H2</div>
                    <div><b>Engine:</b> Native Veyron Desktop Kernel</div>
                  </div>
                </div>`;
            } else if (type === 'personal') {
              content.innerHTML = `
                <h2 style="margin-bottom:12px;">Personalization</h2>
                <div style="margin-bottom:12px;font-weight:500;">Background Wallpaper:</div>
                <div class="wallpaper-preview-grid">
                  ${WALLPAPERS.map((wp, idx) => `
                    <div class="wp-card ${this.currentWpIndex === idx ? 'active' : ''}" style="background-image:url('${wp.url}')" data-wp-idx="${idx}"></div>
                  `).join('')}
                </div>
                <div style="margin-top:24px;">
                  <div style="margin-bottom:10px;font-weight:500;">Choose Your Mode:</div>
                  <button class="tool-btn" data-theme-btn="dark">🌙 Dark Mode</button>
                  <button class="tool-btn" data-theme-btn="light">☀️ Light Mode</button>
                </div>`;
            } else if (type === 'reset') {
              content.innerHTML = `
                <h2 style="margin-bottom:12px;color:#ef4444;">Reset Veyron OS</h2>
                <p style="color:var(--text-muted);margin-bottom:20px;">Restores the system to factory defaults and returns to the initial setup.</p>
                <button class="tool-btn" style="background:#dc2626;color:white;border:none;padding:10px 20px;font-weight:600;" data-settings-reset="true">Reset Everything Now</button>`;
            }
          };
        });
      }

      /* -------------------------------------------------------------
         DOOM 3D Raycaster Canvas Engine
         ------------------------------------------------------------- */
      initDoom(win) {
        const canvas = win.querySelector('#doom-canvas');
        const ctx = canvas.getContext('2d');
        let px = 2.5, py = 2.5, pa = 0;
        const map = [
          [1,1,1,1,1,1,1,1],
          [1,0,0,0,0,0,0,1],
          [1,0,1,0,0,1,0,1],
          [1,0,0,0,0,0,0,1],
          [1,0,1,0,0,1,0,1],
          [1,0,0,0,0,0,0,1],
          [1,1,1,1,1,1,1,1]
        ];

        const keys = {};
        window.addEventListener('keydown', (e) => keys[e.key.toLowerCase()] = true);
        window.addEventListener('keyup', (e) => keys[e.key.toLowerCase()] = false);

        let active = true;
        win.querySelector('.control-btn.close').addEventListener('click', () => active = false);

        const loop = () => {
          if (!active) return;
          if (keys['w'] || keys['arrowup']) { px += Math.cos(pa) * 0.05; py += Math.sin(pa) * 0.05; }
          if (keys['s'] || keys['arrowdown']) { px -= Math.cos(pa) * 0.05; py -= Math.sin(pa) * 0.05; }
          if (keys['a'] || keys['arrowleft']) pa -= 0.04;
          if (keys['d'] || keys['arrowright']) pa += 0.04;

          // Sky & Floor
          ctx.fillStyle = '#202028';
          ctx.fillRect(0, 0, canvas.width, canvas.height / 2);
          ctx.fillStyle = '#3a2e20';
          ctx.fillRect(0, canvas.height / 2, canvas.width, canvas.height / 2);

          // 3D Raycasting
          const numRays = canvas.width;
          const fov = Math.PI / 3;
          for (let i = 0; i < numRays; i++) {
            const rayAngle = (pa - fov / 2) + (i / numRays) * fov;
            let dist = 0;
            let hit = false;
            while (!hit && dist < 12) {
              dist += 0.05;
              const testX = Math.floor(px + Math.cos(rayAngle) * dist);
              const testY = Math.floor(py + Math.sin(rayAngle) * dist);
              if (testX < 0 || testX >= 8 || testY < 0 || testY >= 7 || map[testY][testX] === 1) {
                hit = true;
              }
            }

            const corrected = dist * Math.cos(rayAngle - pa);
            const wallH = Math.min(canvas.height, (canvas.height / corrected) * 1.5);
            const brightness = Math.max(20, Math.floor(255 - dist * 22));
            ctx.fillStyle = `rgb(${brightness}, ${Math.floor(brightness * 0.3)}, ${Math.floor(brightness * 0.3)})`;
            ctx.fillRect(i, (canvas.height - wallH) / 2, 1, wallH);
          }

          requestAnimationFrame(loop);
        };
        requestAnimationFrame(loop);
      }

      /* -------------------------------------------------------------
         Global OS Helpers
         ------------------------------------------------------------- */
      refreshDesktop() {
        VeyronSound.playClick();
        this.showToast('Desktop Refreshed', 'Icons and state reloaded.');
      }

      createFolder() {
        const name = prompt('Enter Folder Name:', 'New Folder');
        if (name) {
          this.fs['C:']['Users']['Default']['Desktop'][name] = {};
          this.showToast('Folder Created', `Created folder "${name}" on Desktop.`);
        }
      }

      saveNotepad() {
        const ta = document.getElementById('notepad-content');
        if (ta) {
          const blob = new Blob([ta.value], { type: 'text/plain' });
          const a = document.createElement('a');
          a.download = 'Document.txt';
          a.href = URL.createObjectURL(blob);
          a.click();
          this.showToast('File Saved', 'Downloaded Document.txt to your computer.');
        }
      }

      // Calculator Methods
      calcNum(n) {
        const main = document.getElementById('calc-val');
        if (main.textContent === '0' && n !== '.') main.textContent = n;
        else main.textContent += n;
      }
      calcClear() {
        document.getElementById('calc-val').textContent = '0';
        document.getElementById('calc-hist').textContent = '';
      }
      calcDel() {
        const main = document.getElementById('calc-val');
        main.textContent = main.textContent.length > 1 ? main.textContent.slice(0, -1) : '0';
      }
      calcOp(op) {
        const main = document.getElementById('calc-val');
        const hist = document.getElementById('calc-hist');
        hist.textContent = `${main.textContent} ${op}`;
        main.textContent = '0';
      }
      
      safeEval(expr) {
        try {
          const parts = expr.trim().split(/\s+/);
          if (parts.length === 3) {
            const a = parseFloat(parts[0]);
            const op = parts[1];
            const b = parseFloat(parts[2]);
            if (!isNaN(a) && !isNaN(b)) {
              switch (op) {
                case '+': return a + b;
                case '-': return a - b;
                case '*': return a * b;
                case '/': return b !== 0 ? a / b : 'Error';
                case '%': return a % b;
              }
            }
          }
          return parseFloat(expr) || 0;
        } catch (e) {
          return 'Error';
        }
      }

      calcEq() {
        const main = document.getElementById('calc-val');
        const hist = document.getElementById('calc-hist');
        try {
          const expr = `${hist.textContent} ${main.textContent}`.replace('×', '*').replace('÷', '/');
          const res = this.safeEval(expr);
          hist.textContent = '';
          main.textContent = res;
        } catch(e) {
          main.textContent = 'Error';
        }
      }
    }

    const Veyron = new OSManager();
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', () => Veyron.init());
    } else {
      Veyron.init();
    }
