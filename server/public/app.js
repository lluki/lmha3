const app = document.getElementById('app');
const userInfo = document.getElementById('user-info');
const mainNav = document.getElementById('main-nav');
const adminTab = document.getElementById('admin-tab');
const adminModal = document.getElementById('admin-modal');
const modalTitle = document.getElementById('modal-title');
const modalContent = document.getElementById('modal-content');
const createBtn = document.getElementById('floating-create-btn');

let currentUser = null;
let activeTab = 'overview';
let healthcheckLoading = false;
let healthcheckResults = null;

async function runHealthcheck() {
    healthcheckLoading = true;
    healthcheckResults = null;
    renderAdmin(); // Refresh UI to show loading state

    try {
        const resp = await fetch('/api/admin/healthcheck');
        if (resp.ok) {
            healthcheckResults = await resp.json();
        } else {
            alert('Healthcheck failed: ' + await resp.text());
        }
    } catch (e) {
        alert('Healthcheck error: ' + e);
    } finally {
        healthcheckLoading = false;
        renderAdmin();
    }
}

window.runHealthcheck = runHealthcheck;

window.showModal = (title, content) => {
    modalTitle.textContent = title;
    modalContent.innerHTML = content;
    adminModal.showModal();
};

window.closeModal = () => {
    adminModal.close();
};

if (createBtn) {
    createBtn.addEventListener('click', () => {
        if (typeof openCreationDialog === 'function') openCreationDialog();
    });
}

async function fetchVersion() {
    try {
        const resp = await fetch('/api/version');
        if (resp.ok) {
            const data = await resp.json();
            document.getElementById('app-version').textContent = data.version;
        }
    } catch (e) {
        console.error('Failed to fetch version:', e);
    }
}

async function checkAuth() {
    fetchVersion();
    try {
        const resp = await fetch('/api/me');
        if (resp.ok) {
            currentUser = await resp.json();
            renderLayout();
        } else {
            renderLogin();
        }
    } catch (e) {
        console.error('Auth check failed:', e);
        renderLogin();
    }
}

function renderLogin(error = '') {
    if (overviewInterval) {
        clearInterval(overviewInterval);
        overviewInterval = null;
    }
    mainNav.classList.add('hidden');
    userInfo.innerHTML = '';
    app.innerHTML = `
        <article style="max-width: 400px; margin: 50px auto;">
            <h2>Login</h2>
            ${error ? `<p style="color: red;">${error}</p>` : ''}
            <form id="login-form">
                <input name="username" placeholder="Username" required autocomplete="username" />
                <input name="password" type="password" placeholder="Password" required autocomplete="current-password" />
                <button type="submit">Login</button>
            </form>
        </article>
    `;

    document.getElementById('login-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const formData = new FormData(e.target);
        const params = new URLSearchParams(formData);
        
        try {
            const resp = await fetch('/api/login', {
                method: 'POST',
                body: params
            });
            if (resp.ok) {
                checkAuth();
            } else {
                renderLogin('Invalid credentials');
            }
        } catch (err) {
            renderLogin('Login failed');
        }
    });
}

function renderLayout() {
    mainNav.classList.remove('hidden');
    
    const banner = document.getElementById('passive-banner');
    if (banner) {
        banner.style.display = currentUser.is_passive ? 'block' : 'none';
    }

    userInfo.innerHTML = ''; 
    
    const userInfoList = document.createElement('li');
    userInfoList.style.display = 'flex';
    userInfoList.style.flexDirection = 'column';
    userInfoList.style.justifyContent = 'center';
    userInfoList.style.lineHeight = '1.2';
    userInfoList.innerHTML = `
        <strong>${currentUser.username}</strong>
        ${!currentUser.is_admin ? '<small id="user-house-name" class="secondary" style="font-size: 0.7rem; opacity: 0.8;"></small>' : ''}
    `;
    userInfo.appendChild(userInfoList);

    const logoutList = document.createElement('li');
    logoutList.innerHTML = `<button id="logout-btn" class="outline secondary" style="margin: 0; padding: 4px 12px; font-size: 0.8rem;">Logout</button>`;
    userInfo.appendChild(logoutList);

    document.getElementById('logout-btn').addEventListener('click', async () => {
        await fetch('/api/logout', { method: 'POST' });
        currentUser = null;
        checkAuth();
    });

    if (!currentUser.is_admin) {
        fetch('/api/houses').then(r => r.json()).then(houses => {
            const h = houses.find(h => h.id === currentUser.house_id);
            if (h) {
                const el = document.getElementById('user-house-name');
                if (el) el.textContent = h.name;
            }
        }).catch(e => console.error(e));
    }

    if (currentUser.is_admin) {
        adminTab.closest('li').classList.remove('hidden');
    } else {
        adminTab.closest('li').classList.add('hidden');
        if (activeTab === 'admin') activeTab = 'overview';
    }

    renderActiveTab();
}

let overviewInterval = null;

function renderActiveTab() {
    if (overviewInterval) {
        clearInterval(overviewInterval);
        overviewInterval = null;
    }

    if (createBtn) {
        createBtn.style.display = activeTab === 'admin' ? 'flex' : 'none';
    }

    // Update tab links
    document.querySelectorAll('.tab-link').forEach(link => {
        if (link.dataset.tab === activeTab) {
            link.classList.add('active');
            link.setAttribute('aria-current', 'page');
        } else {
            link.classList.remove('active');
            link.removeAttribute('aria-current');
        }
    });

    switch (activeTab) {
        case 'overview':
            renderOverview();
            overviewInterval = setInterval(renderOverview, 10000);
            break;
        case 'history':
            renderHistory();
            break;
        case 'logs':
            renderLogs();
            break;
        case 'admin':
            renderAdmin();
            break;
    }
}

// Add tab switching event listeners
document.querySelectorAll('.tab-link').forEach(link => {
    link.addEventListener('click', (e) => {
        e.preventDefault();
        activeTab = link.dataset.tab;
        renderActiveTab();
    });
});

async function renderOverview() {
    if (!document.getElementById('stats-container')) {
        app.innerHTML = `
            <div id="stats-container" class="grid" style="margin-bottom: 2rem;">
                <article style="padding: 1rem; margin-bottom: 0;">
                    <header style="padding: 0.5rem 1rem; margin-bottom: 0.5rem; display: flex; justify-content: space-between; align-items: center;">
                        <small>Panel Production</small>
                        <small id="pv-time" class="secondary" style="font-size: 0.7rem;"></small>
                    </header>
                    <h3 id="pv-value" style="margin: 0; color: var(--pico-primary);">-- kW</h3>
                </article>
                <article style="padding: 1rem; margin-bottom: 0;">
                    <header style="padding: 0.5rem 1rem; margin-bottom: 0.5rem; display: flex; justify-content: space-between; align-items: center;">
                        <small>Household Consumption</small>
                        <small id="cons-time" class="secondary" style="font-size: 0.7rem;"></small>
                    </header>
                    <h3 id="consumption-value" style="margin: 0; color: var(--pico-del-color);">-- kW</h3>
                </article>
            </div>
            <article>
                <header style="display: flex; justify-content: space-between; align-items: center;">
                    <strong>${currentUser.is_admin ? 'All Devices' : 'Your Devices'}</strong>
                </header>
                <div id="overview-content" aria-busy="true">Loading...</div>
            </article>
        `;
    }

    try {
        const fetchTasks = [
            fetch('/api/devices'),
            fetch('/api/metrics'),
            fetch('/api/history'),
            fetch('/api/houses')
        ];
        if (currentUser.is_admin) {
            fetchTasks.push(fetch('/api/tenants'));
        }

        const responses = await Promise.all(fetchTasks);
        const devices = await responses[0].json();
        const metrics = await responses[1].json();
        const history = await responses[2].json();
        const houses = await responses[3].json();
        const tenants = currentUser.is_admin ? await responses[4].json() : [];

        // Update Stats
        if (metrics) {
            document.getElementById('pv-value').textContent = `${(metrics.pv / 1000).toFixed(1)} kW`;
            document.getElementById('consumption-value').textContent = `${(metrics.consumption / 1000).toFixed(1)} kW`;
        }

        const tenantMap = {};
        tenants.forEach(t => tenantMap[t.id] = t.username);

        const userDevices = currentUser.is_admin ? devices : devices.filter(d => d.tenant_id === currentUser.tenant_id);
        const content = document.getElementById('overview-content');
        if (!content) return;
        content.removeAttribute('aria-busy');

        if (userDevices.length === 0) {
            content.innerHTML = '<p>No devices found.</p>';
            return;
        }

        let html = '<table><thead><tr><th>Name</th>';
        if (currentUser.is_admin) html += '<th>Owner</th>';
        html += '<th>Mode</th><th>Status</th><th>Last Seen</th><th>Action</th></tr></thead><tbody>';

        for (const d of userDevices) {
            // Offline and Syncing detection
            const now = Date.now();
            const lastFeedback = d.last_feedback_time ? new Date(d.last_feedback_time).getTime() : 0;
            const lastRequest = d.last_request_time ? new Date(d.last_request_time).getTime() : 0;
            const isOffline = (lastRequest > lastFeedback + 20000) || (now - lastFeedback > 300000);
            const isSyncing = d.desired_state !== d.current_state;

            let lastSeen = d.last_feedback_time ? new Date(d.last_feedback_time).toLocaleString() : 'Never';
            let statusText = `<code>${d.current_state}</code>`;
            if (isSyncing) statusText += ` <small class="syncing-indicator" style="display: block; font-size: 0.65rem; color: var(--pico-ins-color);">Syncing...</small>`;
            if (isOffline) statusText += ` <span class="offline-badge" style="background: var(--pico-del-color); color: white; padding: 1px 4px; border-radius: 3px; font-size: 0.6rem; font-weight: bold; margin-left: 4px; vertical-align: middle;">OFFLINE</span>`;
            
            let modeText = '';

            const schObj = d.scheduling_type;
            const schType = (schObj && typeof schObj === 'object' ? schObj.type : schObj) || 'UNKNOWN';
            const untilTime = (schObj && schObj.until) ? new Date(schObj.until).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) : '';
            
            let until = '';
            if (schObj && schObj.until) {
                const date = new Date(schObj.until);
                const offset = date.getTimezoneOffset() * 60000;
                until = new Date(date.getTime() - offset).toISOString().slice(0, 16);
            }

            if (schType === 'BOILER') {
                const hours = Math.floor(d.runtime_24h / 60);
                const mins = d.runtime_24h % 60;
                statusText += `<br><small class="secondary">24h Runtime: ${hours}h ${mins}m</small>`;
            } else if (d.current_state === 'ON') {
                const lastOn = history.find(t => t.device_id === d.id && t.source === 'DEVICE_STATE' && t.value === 1.0);
                if (lastOn) {
                    const diff = Date.now() - new Date(lastOn.timestamp).getTime();
                    const mins = Math.floor(diff / 60000);
                    if (mins < 60) {
                        statusText += ` <small>(on ${mins}m)</small>`;
                    } else {
                        statusText += ` <small>(since ${new Date(lastOn.timestamp).toLocaleTimeString()})</small>`;
                    }
                }
            } else if (d.current_state === 'OFF') {
                const lastOffIndex = history.findIndex(t => t.device_id === d.id && t.source === 'DEVICE_STATE' && t.value === 0.0);
                if (lastOffIndex !== -1) {
                    const lastOff = history[lastOffIndex];
                    const lastOn = history.slice(lastOffIndex + 1).find(t => t.device_id === d.id && t.source === 'DEVICE_STATE' && t.value === 1.0);
                    
                    if (lastOn) {
                        const start = new Date(lastOn.timestamp);
                        const end = new Date(lastOff.timestamp);
                        const diffMs = end - start;
                        const hours = Math.floor(diffMs / 3600000);
                        const mins = Math.floor((diffMs % 3600000) / 60000);
                        const durationStr = hours > 0 ? `${hours}h ${mins}m` : `${mins}m`;
                        
                        const timeOpts = { hour: '2-digit', minute: '2-digit' };
                        statusText += `<br><small class="secondary" style="font-size: 0.75rem; white-space: nowrap;">Last: ${start.toLocaleTimeString([], timeOpts)} - ${end.toLocaleTimeString([], timeOpts)} (${durationStr})</small>`;
                    }
                }
            }

            html += `
                <tr>
                    <td data-label="Name">${d.name}</td>`;
            
            if (currentUser.is_admin) {
                const ownerName = tenantMap[d.tenant_id] || 'Unknown';
                html += `<td data-label="Owner">${ownerName}</td>`;
            }

            html += `
                    <td data-label="Config">
                        <label style="font-size: 0.7rem; margin-bottom: 0.2rem;">Load (W)
                            <input type="number" value="${d.expected_load}" id="ov-load-${d.id}" style="margin-bottom:0.4rem; font-size: 0.8rem; padding: 2px 8px;">
                        </label>
                        <div id="ov-boiler-config-${d.id}" style="${schType === 'BOILER' ? '' : 'display:none'}">
                            <label style="font-size: 0.7rem; margin-bottom: 0.2rem;">Charge (Days)
                                <input type="number" min="1" max="8" value="${d.full_charge_n_day}" id="ov-full-${d.id}" style="margin-bottom:0.4rem; font-size: 0.8rem; padding: 2px 8px;">
                            </label>
                            <label style="font-size: 0.7rem; margin-bottom: 0.2rem;">Min (Mins)
                                <input type="number" min="0" value="${d.min_daily_charge}" id="ov-min-${d.id}" style="margin-bottom:0; font-size: 0.8rem; padding: 2px 8px;">
                            </label>
                        </div>
                    </td>
                    <td data-label="Scheduling">
                        <label style="font-size: 0.7rem; margin-bottom: 0.2rem;">Mode
                            <select id="ov-sch-${d.id}" onchange="handleSchedulingChangeOverview('${d.id}', this.value)" style="margin-bottom:0.4rem; font-size: 0.8rem; padding: 2px 8px; height: auto;">
                                <option value="BOILER" ${schType === 'BOILER' ? 'selected' : ''}>Boiler</option>
                                <option value="NONE" ${schType === 'NONE' ? 'selected' : ''}>Manual</option>
                                <option value="FORCE_ON" ${schType === 'FORCE_ON' ? 'selected' : ''}>Force ON</option>
                                <option value="FORCE_OFF" ${schType === 'FORCE_OFF' ? 'selected' : ''}>Force OFF</option>
                            </select>
                        </label>
                        <div id="ov-until-container-${d.id}" style="${(schType === 'FORCE_ON' || schType === 'FORCE_OFF') ? '' : 'display:none'}">
                            <label style="font-size: 0.7rem; margin-bottom: 0.2rem;">Until
                                <input type="datetime-local" value="${until}" id="ov-until-${d.id}" style="margin-bottom:0; font-size: 0.8rem; padding: 2px 8px;">
                            </label>
                        </div>
                    </td>
                    <td data-label="Status">${statusText}</td>
                    <td data-label="Last Seen">${lastSeen}</td>
                    <td data-label="Action">
                        <button class="outline" style="margin-bottom:0.5rem; padding: 2px 8px; width: 100%; font-size: 0.8rem;" onclick="toggleDevice('${d.id}')">Toggle</button>
                        <button class="outline contrast" style="margin:0; padding: 2px 8px; width: 100%; font-size: 0.8rem;" onclick="updateDeviceConfigOverview('${d.id}')">Save</button>
                    </td>
                </tr>
            `;
        }
        html += '</tbody></table>';
        content.innerHTML = html;
    } catch (e) {
        const content = document.getElementById('overview-content');
        if (content) {
            content.removeAttribute('aria-busy');
            content.innerHTML = `<p style="color: red;">${e.message}</p>`;
        }
    }
}

function renderHistory() {
    app.innerHTML = `
        <article>
            <header style="display: flex; justify-content: space-between; align-items: center;">
                <strong>Event History</strong>
                <fieldset style="margin: 0; padding: 0; border: none; display: flex; align-items: center;">
                    <label style="margin: 0; display: flex; align-items: center; cursor: pointer;">
                        <input type="checkbox" id="show-consumption" style="margin-right: 8px;" checked />
                        <span>Show All Telemetry</span>
                    </label>
                </fieldset>
            </header>
            <div id="history-content" aria-busy="true">Loading...</div>
        </article>
    `;
    
    const showConsumption = document.getElementById('show-consumption');
    showConsumption.addEventListener('change', () => fetchAndRenderHistory(showConsumption.checked));
    
    fetchAndRenderHistory(true);
}

async function fetchAndRenderHistory(includeAll) {
    const content = document.getElementById('history-content');
    if (!content) return;
    content.setAttribute('aria-busy', 'true');
    
    try {
        const [historyResp, devicesResp, tenantsResp] = await Promise.all([
            fetch(`/api/history?events_only=${!includeAll}`),
            fetch('/api/devices'),
            fetch('/api/tenants')
        ]);
        const history = await historyResp.json();
        const devices = await devicesResp.json();
        const tenants = await tenantsResp.json();
        
        const tenantMap = {};
        tenants.forEach(t => tenantMap[t.id] = t.username);

        content.removeAttribute('aria-busy');
        
        let filtered = history;
        
        if (filtered.length === 0) {
            content.innerHTML = '<p>No history found.</p>';
            return;
        }

        let html = '<table><thead><tr><th>Time</th><th>Source</th><th>Event</th><th>Value</th></tr></thead><tbody>';
        for (const t of filtered) {
            const device = devices.find(d => d.id === t.device_id);
            let sourceName = 'System';
            if (device) {
                const ownerName = tenantMap[device.tenant_id] || 'unknown';
                sourceName = `${device.name} <small class="secondary">(${ownerName})</small>`;
            }
            let eventType = t.source;
            let valText = t.value;
            
            if (t.source === 'DEVICE_STATE') {
                eventType = 'State';
                valText = t.value === 1.0 ? '<mark style="background: var(--pico-ins-color); color: white; padding: 2px 6px; border-radius: 4px;">ON</mark>' : '<mark style="background: var(--pico-del-color); color: white; padding: 2px 6px; border-radius: 4px;">OFF</mark>';
            } else if (t.source === 'DEVICE_CONSUMPTION') {
                eventType = 'Device Load';
                valText = `<code>${(t.value / 1000).toFixed(1)} kW</code>`;
            } else if (t.source === 'PV_PRODUCTION') {
                eventType = 'Panel Production';
                valText = `<code>${(t.value / 1000).toFixed(1)} kW</code>`;
            } else if (t.source === 'HOUSE_CONSUMPTION') {
                eventType = 'House Total Load';
                valText = `<code>${(t.value / 1000).toFixed(1)} kW</code>`;
            }
            
            html += `
                <tr>
                    <td>${new Date(t.timestamp).toLocaleTimeString()} <small class="secondary">${new Date(t.timestamp).toLocaleDateString()}</small></td>
                    <td>${sourceName}</td>
                    <td>${eventType}</td>
                    <td>${valText}</td>
                </tr>
            `;
        }
        html += '</tbody></table>';
        if (history.length >= 100) {
            html += '<p style="text-align: center;"><small class="secondary">More events hidden. Showing last 100.</small></p>';
        }
        content.innerHTML = html;
    } catch (e) {
        content.removeAttribute('aria-busy');
        content.innerHTML = `<p style="color: red;">${e.message}</p>`;
    }
}

async function renderLogs() {
    app.innerHTML = `
        <article>
            <header style="display: flex; justify-content: space-between; align-items: center;">
                <strong>System Logs</strong>
                <select id="log-level-filter" style="width: auto; margin: 0;">
                    <option value="ALL">All Levels</option>
                    <option value="TRACE">Trace+</option>
                    <option value="INFO">Info+</option>
                    <option value="WARN">Warn+</option>
                    <option value="ERROR">Error</option>
                </select>
            </header>
            <div id="logs-content" aria-busy="true">Loading...</div>
        </article>
    `;
    
    document.getElementById('log-level-filter').addEventListener('change', (e) => {
        fetchAndRenderLogs('logs-content', e.target.value);
    });
    
    fetchAndRenderLogs('logs-content', 'ALL');
}

async function fetchAndRenderLogs(elementId, levelFilter = 'ALL') {
    const content = document.getElementById(elementId);
    content.setAttribute('aria-busy', 'true');
    try {
        const resp = await fetch('/api/logs');
        if (!resp.ok) throw new Error('Failed to fetch logs');
        let logs = await resp.json();
        
        // Filter logic
        if (levelFilter !== 'ALL') {
            logs = logs.filter(l => {
                if (levelFilter === 'ERROR') return l.level === 'ERROR';
                if (levelFilter === 'WARN') return l.level === 'WARN' || l.level === 'ERROR';
                if (levelFilter === 'INFO') return l.level === 'INFO' || l.level === 'WARN' || l.level === 'ERROR';
                if (levelFilter === 'TRACE') return l.level === 'TRACE' || l.level === 'INFO' || l.level === 'WARN' || l.level === 'ERROR';
                return true;
            });
        }
        
        content.removeAttribute('aria-busy');
        if (logs.length === 0) {
            content.innerHTML = '<p>No logs available.</p>';
            return;
        }

        let html = '<div style="max-height: 500px; overflow-y: auto; font-family: monospace; font-size: 0.85rem; padding: 1rem; background: var(--pico-card-background-color); border-radius: var(--pico-border-radius);">';
        logs.reverse().forEach(log => {
            let color = 'var(--pico-color)';
            if (log.level === 'ERROR') color = 'var(--pico-del-color)';
            else if (log.level === 'WARN') color = 'var(--pico-ins-color)';
            
            html += `
                <div style="margin-bottom: 4px; color: ${color};">
                    [${new Date(log.timestamp).toLocaleTimeString()}] <strong>${log.level}</strong> [${log.target}] ${log.message}
                </div>
            `;
        });
        html += '</div>';
        content.innerHTML = html;
    } catch (e) {
        content.removeAttribute('aria-busy');
        content.innerHTML = `<p style="color: red;">${e.message}</p>`;
    }
}

function renderTenantCard(t, houseName) {
    return `
        <article class="summary-card" onclick="openTenantDetails('${t.id}')">
            <header>
                <strong>${t.username}</strong>
                <span class="secondary" style="font-size: 0.8rem;">${t.is_admin ? 'Admin' : 'User'}</span>
            </header>
            <div class="card-body">
                <div>House: ${houseName}</div>
            </div>
            <div class="card-footer">
                Click for details
            </div>
        </article>
    `;
}

async function renderTenantDetails(id, isEdit = false) {
    try {
        const [tenantsResp, housesResp] = await Promise.all([
            fetch('/api/tenants'),
            fetch('/api/houses')
        ]);
        const tenants = await tenantsResp.json();
        const houses = await housesResp.json();
        const t = tenants.find(x => x.id === id);
        if (!t) return;

        const houseName = houses.find(h => h.id === t.house_id)?.name || 'Unknown';

        let content = '';
        if (isEdit) {
            const hOpts = houses.map(h => `<option value="${h.id}" ${h.id === t.house_id ? 'selected' : ''}>${h.name}</option>`).join('');
            content = `
                <form id="edit-tenant-form">
                    <label>Username <input name="username" value="${t.username}" required /></label>
                    <label>House 
                        <select name="house_id" required>${hOpts}</select>
                    </label>
                    <label>
                        <input type="checkbox" name="is_admin" value="true" ${t.is_admin ? 'checked' : ''}> Admin Access
                    </label>
                    <label>New Password <input name="password" type="password" placeholder="Leave empty to keep" /></label>
                    <div class="grid">
                        <button type="submit">Save Changes</button>
                        <button type="button" class="secondary" onclick="renderTenantDetails('${t.id}', false)">Cancel</button>
                    </div>
                </form>
            `;
        } else {
            content = `
                <div class="detail-row"><span class="detail-label">Username</span><span>${t.username}</span></div>
                <div class="detail-row"><span class="detail-label">House</span><span>${houseName}</span></div>
                <div class="detail-row"><span class="detail-label">Role</span><span>${t.is_admin ? 'Administrator' : 'Tenant'}</span></div>
                <div class="grid" style="margin-top: 2rem;">
                    <button onclick="renderTenantDetails('${t.id}', true)">Edit</button>
                    <button class="secondary" onclick="deleteTenant('${t.id}')">Delete</button>
                </div>
            `;
        }

        showModal(`User: ${t.username}`, content);

        if (isEdit) {
            document.getElementById('edit-tenant-form').addEventListener('submit', async (e) => {
                e.preventDefault();
                const formData = new FormData(e.target);
                const username = formData.get('username');
                const house_id = formData.get('house_id');
                const is_admin = formData.get('is_admin') === 'true';
                const password = formData.get('password');
                
                const params = new URLSearchParams({ username, house_id, is_admin });
                if (password) params.append('password', password);
                
                const resp = await fetch(`/api/tenants/${t.id}`, {
                    method: 'PATCH',
                    body: params
                });
                if (resp.ok) {
                    closeModal();
                    renderAdmin();
                } else {
                    alert('Update failed: ' + await resp.text());
                }
            });
        }
    } catch (e) {
        console.error('Failed to render tenant details:', e);
    }
}

window.openTenantDetails = (id) => renderTenantDetails(id, false);

function renderDeviceCard(d, tenantName) {
    const schObj = d.scheduling_type;
    const schType = (schObj && typeof schObj === 'object' ? schObj.type : schObj) || 'UNKNOWN';
    
    // Offline and Syncing detection
    const now = Date.now();
    const lastFeedback = d.last_feedback_time ? new Date(d.last_feedback_time).getTime() : 0;
    const lastRequest = d.last_request_time ? new Date(d.last_request_time).getTime() : 0;
    const isOffline = (lastRequest > lastFeedback + 20000) || (now - lastFeedback > 300000);
    const isSyncing = d.desired_state !== d.current_state;

    return `
        <article class="summary-card" onclick="openDeviceDetails('${d.id}')">
            <header>
                <div style="display: flex; justify-content: space-between; align-items: flex-start; width: 100%;">
                    <strong>${d.name}</strong>
                    ${isOffline ? '<span style="background: var(--pico-del-color); color: white; padding: 1px 4px; border-radius: 3px; font-size: 0.6rem; font-weight: bold;">OFFLINE</span>' : ''}
                </div>
                <span class="secondary" style="font-size: 0.8rem;">${schType} ${isSyncing ? '<small style="color: var(--pico-ins-color)">(Syncing...)</small>' : ''}</span>
            </header>
            <div class="card-body">
                <div>Owner: ${tenantName}</div>
                <div>Topic: ${d.mqtt_topic}</div>
            </div>
            <div class="card-footer">
                Click for details
            </div>
        </article>
    `;
}

async function renderDeviceDetails(id, isEdit = false) {
    try {
        const [tenantsResp, devicesResp] = await Promise.all([
            fetch('/api/tenants'),
            fetch('/api/devices')
        ]);
        const tenants = await tenantsResp.json();
        const devices = await devicesResp.json();
        const d = devices.find(x => x.id === id);
        if (!d) return;

        const tenantName = tenants.find(t => t.id === d.tenant_id)?.username || 'Unknown';
        const schObj = d.scheduling_type;
        const schType = (schObj && typeof schObj === 'object' ? schObj.type : schObj) || 'UNKNOWN';
        
        let until = '';
        if (schObj && schObj.until) {
            const date = new Date(schObj.until);
            const offset = date.getTimezoneOffset() * 60000;
            until = new Date(date.getTime() - offset).toISOString().slice(0, 16);
        }

        let content = '';
        if (isEdit) {
            const tOpts = tenants.map(t => `<option value="${t.id}" ${t.id === d.tenant_id ? 'selected' : ''}>${t.username}</option>`).join('');
            content = `
                <form id="edit-device-form">
                    <div class="grid">
                        <label>Name <input name="name" value="${d.name}" required /></label>
                        <label>MQTT Topic <input name="mqtt_topic" value="${d.mqtt_topic}" required /></label>
                    </div>
                    <label>Owner (Tenant)
                        <select name="tenant_id" required>${tOpts}</select>
                    </label>
                    <div class="grid">
                        <label>Load (W) <input type="number" name="expected_load" value="${d.expected_load}" required /></label>
                        <label>Mode
                            <select name="scheduling_type" id="edit-d-sch" onchange="handleSchedulingChangeInModal(this.value)">
                                <option value="BOILER" ${schType === 'BOILER' ? 'selected' : ''}>Boiler</option>
                                <option value="NONE" ${schType === 'NONE' ? 'selected' : ''}>Manual</option>
                                <option value="FORCE_ON" ${schType === 'FORCE_ON' ? 'selected' : ''}>Force ON</option>
                                <option value="FORCE_OFF" ${schType === 'FORCE_OFF' ? 'selected' : ''}>Force OFF</option>
                            </select>
                        </label>
                    </div>
                    <div id="modal-boiler-config" style="${schType === 'BOILER' ? '' : 'display:none'}">
                        <div class="grid">
                            <label>Charge (Days) <input type="number" name="full_charge_n_day" min="1" max="8" value="${d.full_charge_n_day}" /></label>
                            <label>Min (Mins) <input type="number" name="min_daily_charge" min="0" value="${d.min_daily_charge}" /></label>
                        </div>
                    </div>
                    <div id="modal-until-container" style="${(schType === 'FORCE_ON' || schType === 'FORCE_OFF') ? '' : 'display:none'}">
                        <label>Until <input type="datetime-local" name="scheduling_until" value="${until}" /></label>
                    </div>
                    <div class="grid" style="margin-top: 2rem;">
                        <button type="submit">Save All</button>
                        <button type="button" class="secondary" onclick="renderDeviceDetails('${d.id}', false)">Cancel</button>
                    </div>
                </form>
            `;
        } else {
            const lastFeedback = d.last_feedback_time ? new Date(d.last_feedback_time).toLocaleString() : 'Never';
            const lastRequest = d.last_request_time ? new Date(d.last_request_time).toLocaleString() : 'Never';
            const isSyncing = d.desired_state !== d.current_state;

            content = `
                <div class="detail-row">
                    <span class="detail-label">Observed State</span>
                    <span style="text-align: right;">
                        <code>${d.current_state}</code><br>
                        <small class="secondary">Last Feedback: ${lastFeedback}</small>
                    </span>
                </div>
                <div class="detail-row">
                    <span class="detail-label">Requested State</span>
                    <span style="text-align: right;">
                        <code>${d.desired_state}</code> ${isSyncing ? '<small style="color: var(--pico-ins-color); font-weight: bold;">(PENDING)</small>' : ''}<br>
                        <small class="secondary">Last Request: ${lastRequest}</small>
                    </span>
                </div>
                <hr>
                <div class="detail-row"><span class="detail-label">MQTT Topic</span><span>${d.mqtt_topic}</span></div>
                <div class="detail-row"><span class="detail-label">Owner</span><span>${tenantName}</span></div>
                <div class="detail-row"><span class="detail-label">Load</span><span>${d.expected_load} W</span></div>
                <div class="detail-row"><span class="detail-label">Scheduling Mode</span><span>${schType}</span></div>
                ${until ? `<div class="detail-row"><span class="detail-label">Mode Until</span><span>${new Date(until).toLocaleString()}</span></div>` : ''}
                ${schType === 'BOILER' ? `
                    <div class="detail-row"><span class="detail-label">Charge Interval</span><span>${d.full_charge_n_day} days</span></div>
                    <div class="detail-row"><span class="detail-label">Daily Min</span><span>${d.min_daily_charge} mins</span></div>
                ` : ''}
                <div class="grid" style="margin-top: 2rem;">
                    <button onclick="renderDeviceDetails('${d.id}', true)">Edit Config</button>
                    <button class="secondary" onclick="deleteDevice('${d.id}')">Delete Device</button>
                </div>
            `;
        }

        showModal(`Device: ${d.name}`, content);

        if (isEdit) {
            document.getElementById('edit-device-form').addEventListener('submit', async (e) => {
                e.preventDefault();
                const formData = new FormData(e.target);
                const id = d.id;
                
                // Helper to gather all inputs and call updateDeviceConfigAdmin
                const body = {
                    name: formData.get('name'),
                    mqtt_topic: formData.get('mqtt_topic'),
                    tenant_id: formData.get('tenant_id'),
                    expected_load: parseInt(formData.get('expected_load')),
                    full_charge_n_day: parseInt(formData.get('full_charge_n_day') || 0),
                    min_daily_charge: parseInt(formData.get('min_daily_charge') || 0),
                    scheduling_type: {}
                };

                const type = formData.get('scheduling_type');
                if (type === 'FORCE_ON' || type === 'FORCE_OFF') {
                    body.scheduling_type = { type: type, until: new Date(formData.get('scheduling_until')).toISOString() };
                } else {
                    body.scheduling_type = { type: type };
                }

                const resp = await fetch(`/api/devices/${id}`, {
                    method: 'PATCH',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(body)
                });
                
                if (resp.ok) {
                    closeModal();
                    renderAdmin();
                } else {
                    alert('Update failed: ' + await resp.text());
                }
            });
        }
    } catch (e) {
        console.error('Failed to render device details:', e);
    }
}

window.openDeviceDetails = (id) => renderDeviceDetails(id, false);

window.openCreationDialog = () => {
    const content = `
        <div class="grid">
            <button class="outline" onclick="renderCreateHouseForm()">New House</button>
            <button class="outline" onclick="renderCreateTenantForm()">New User</button>
            <button class="outline" onclick="renderCreateDeviceForm()">New Device</button>
        </div>
    `;
    showModal('Create New Entity', content);
};

window.renderCreateHouseForm = () => {
    const content = `
        <form id="modal-create-house-form">
            <label>House Name <input name="name" required autocomplete="off" /></label>
            <label>HA Host <input name="ha_host" placeholder="192.168.1.100" required /></label>
            <label>HA Token <input name="ha_token" required autocomplete="off" /></label>
            <button type="submit">Create House</button>
        </form>
    `;
    showModal('Create New House', content);
    document.getElementById('modal-create-house-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const formData = new FormData(e.target);
        const params = new URLSearchParams(formData);
        try {
            const resp = await fetch('/api/houses', { method: 'POST', body: params });
            if (resp.ok) { closeModal(); renderAdmin(); }
            else alert('Failed to create house: ' + await resp.text());
        } catch (err) { alert('Error: ' + err); }
    });
};

window.renderCreateTenantForm = async () => {
    try {
        const resp = await fetch('/api/houses');
        const houses = await resp.json();
        const hOpts = houses.map(h => `<option value="${h.id}" ${h.id === currentUser.house_id ? 'selected' : ''}>${h.name}</option>`).join('');
        
        const content = `
            <form id="modal-create-tenant-form">
                <label>Username <input name="username" required autocomplete="off" /></label>
                <label>Password <input name="password" type="password" placeholder="(default: username)" /></label>
                <label>House 
                    <select name="house_id" required>${hOpts}</select>
                </label>
                <label>
                    <input type="checkbox" name="is_admin" value="true"> Admin Access
                </label>
                <button type="submit">Create User</button>
            </form>
        `;
        showModal('Create New User', content);
        document.getElementById('modal-create-tenant-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const formData = new FormData(e.target);
            const params = new URLSearchParams(formData);
            try {
                const resp = await fetch('/api/tenants', { method: 'POST', body: params });
                if (resp.ok) { closeModal(); renderAdmin(); }
                else alert('Failed to create user: ' + await resp.text());
            } catch (err) { alert('Error: ' + err); }
        });
    } catch (err) { alert('Error: ' + err); }
};

window.renderCreateDeviceForm = async () => {
    try {
        const [tenantsResp, discoveryResp] = await Promise.all([
            fetch('/api/tenants'),
            fetch('/api/admin/discover-devices')
        ]);
        const tenants = await tenantsResp.json();
        const discovered = await discoveryResp.json();
        const tOpts = tenants.map(t => `<option value="${t.id}">${t.username}</option>`).join('');
        
        const content = `
            <form id="modal-create-device-form">
                <div class="grid">
                    <label>Name <input name="name" value="Boiler" required /></label>
                    <label>MQTT Topic <input name="mqtt_topic" id="modal-device-topic-input" placeholder="shellypro1-123456" required /></label>
                </div>
                <div id="modal-discovery-container" style="${discovered && discovered.length > 0 ? '' : 'display: none'}; margin-bottom: 1rem;">
                    <small class="secondary">Discovered IDs: </small>
                    <span id="modal-discovered-topics">
                        ${discovered.map(t => `<a href="#" onclick="document.getElementById('modal-device-topic-input').value='${t}'; return false;" style="margin-right: 8px; font-size: 0.8rem;">${t}</a>`).join('')}
                    </span>
                </div>
                <label>Owner (Tenant)
                    <select name="tenant_id" required>
                        <option value="" disabled selected>Select a tenant...</option>
                        ${tOpts}
                    </select>
                </label>
                <button type="submit">Create Device</button>
            </form>
        `;
        showModal('Create New Device', content);
        document.getElementById('modal-create-device-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const formData = new FormData(e.target);
            const params = new URLSearchParams(formData);
            try {
                const resp = await fetch('/api/devices', { method: 'POST', body: params });
                if (resp.ok) { closeModal(); renderAdmin(); }
                else alert('Failed to create device: ' + await resp.text());
            } catch (err) { alert('Error: ' + err); }
        });
    } catch (err) { alert('Error: ' + err); }
};

window.handleSchedulingChangeInModal = (type) => {
    const untilContainer = document.getElementById('modal-until-container');
    const boilerConfig = document.getElementById('modal-boiler-config');
    
    if (type === 'FORCE_ON' || type === 'FORCE_OFF') {
        untilContainer.style.display = 'block';
        boilerConfig.style.display = 'none';
        const input = untilContainer.querySelector('input');
        if (!input.value) {
            const inOneHour = new Date(Date.now() + 3600000);
            const offset = inOneHour.getTimezoneOffset() * 60000;
            input.value = new Date(inOneHour.getTime() - offset).toISOString().slice(0, 16);
        }
    } else if (type === 'BOILER') {
        untilContainer.style.display = 'none';
        boilerConfig.style.display = 'block';
    } else {
        untilContainer.style.display = 'none';
        boilerConfig.style.display = 'none';
    }
};

function renderHouseCard(h) {
    return `
        <article class="summary-card" onclick="openHouseDetails('${h.id}')">
            <header>
                <strong>${h.name}</strong>
                <span class="secondary" style="font-size: 0.8rem;">House</span>
            </header>
            <div class="card-body">
                <div>Host: ${h.ha_host}</div>
            </div>
            <div class="card-footer">
                Click for details
            </div>
        </article>
    `;
}

async function renderHouseDetails(id, isEdit = false) {
    try {
        const resp = await fetch('/api/houses');
        const houses = await resp.json();
        const h = houses.find(x => x.id === id);
        if (!h) return;

        let content = '';
        if (isEdit) {
            content = `
                <form id="edit-house-form">
                    <label>House Name <input name="name" value="${h.name}" required /></label>
                    <label>HA Host <input name="ha_host" value="${h.ha_host}" required /></label>
                    <label>HA Token <input name="ha_token" value="${h.ha_token}" required /></label>
                    <div class="grid">
                        <button type="submit">Save Changes</button>
                        <button type="button" class="secondary" onclick="renderHouseDetails('${h.id}', false)">Cancel</button>
                    </div>
                </form>
            `;
        } else {
            content = `
                <div class="detail-row"><span class="detail-label">Name</span><span>${h.name}</span></div>
                <div class="detail-row"><span class="detail-label">HA Host</span><span>${h.ha_host}</span></div>
                <div class="detail-row"><span class="detail-label">HA Token</span><span>••••••••</span></div>
                <div class="grid" style="margin-top: 2rem;">
                    <button onclick="renderHouseDetails('${h.id}', true)">Edit</button>
                    <button class="secondary" onclick="deleteHouse('${h.id}')">Delete</button>
                </div>
            `;
        }

        showModal(`House: ${h.name}`, content);

        if (isEdit) {
            document.getElementById('edit-house-form').addEventListener('submit', async (e) => {
                e.preventDefault();
                const formData = new FormData(e.target);
                await updateHouse(h.id, formData.get('name'), formData.get('ha_host'), formData.get('ha_token'));
                closeModal();
                renderAdmin();
            });
        }
    } catch (e) {
        console.error('Failed to render house details:', e);
    }
}

window.openHouseDetails = (id) => renderHouseDetails(id, false);

async function renderAdmin() {
    let healthcheckHtml = `
        <section class="admin-section">
            <header style="display: flex; justify-content: space-between; align-items: center;">
                <h3>System Health</h3>
                <button class="outline" onclick="runHealthcheck()" ${healthcheckLoading ? 'disabled aria-busy="true"' : ''}>
                    ${healthcheckLoading ? 'Running Healthcheck...' : 'Run Healthcheck'}
                </button>
            </header>
            <div id="admin-healthcheck-results">
                ${healthcheckResults ? `
                    <div style="background: var(--pico-card-background-color); padding: 1.5rem; border-radius: var(--pico-border-radius); border: 1px solid var(--pico-card-border-color);">
                        
                        <div style="margin-bottom: 1.5rem;">
                            <h4 style="font-size: 1rem; margin-bottom: 0.5rem; display: flex; align-items: center;">
                                <span style="margin-right: 0.5rem;">${healthcheckResults.pv.status === 'ok' ? '✅' : '❌'}</span>
                                PV Sources (${healthcheckResults.pv.message})
                            </h4>
                            <ul style="list-style: none; padding-left: 1.5rem; font-size: 0.85rem; margin: 0;">
                                ${healthcheckResults.pv.details.map(d => `
                                    <li style="margin-bottom: 0.2rem; color: ${d.status === 'ok' ? 'inherit' : 'var(--pico-del-color)'}">
                                        ${d.status === 'ok' ? '✓' : '✗'} <strong>${d.house}</strong>: ${d.status === 'ok' ? 'Connected' : `Error: ${d.message}`}
                                    </li>
                                `).join('')}
                            </ul>
                        </div>

                        <div style="margin-bottom: 1.5rem;">
                            <h4 style="font-size: 1rem; margin-bottom: 0.5rem; display: flex; align-items: center;">
                                <span style="margin-right: 0.5rem;">${healthcheckResults.mqtt.status === 'ok' ? '✅' : '❌'}</span>
                                MQTT Broker
                            </h4>
                            <p style="padding-left: 1.5rem; font-size: 0.85rem; margin: 0;">${healthcheckResults.mqtt.message}</p>
                        </div>

                        <div>
                            <h4 style="font-size: 1rem; margin-bottom: 0.5rem; display: flex; align-items: center;">
                                <span style="margin-right: 0.5rem;">${healthcheckResults.devices.status === 'ok' ? '✅' : '❌'}</span>
                                Devices (${healthcheckResults.devices.message})
                            </h4>
                            <ul style="list-style: none; padding-left: 1.5rem; font-size: 0.85rem; margin: 0;">
                                ${healthcheckResults.devices.details.map(d => `
                                    <li style="margin-bottom: 0.2rem; color: ${d.status === 'ok' ? 'inherit' : 'var(--pico-del-color)'}">
                                        ${d.status === 'ok' ? '✓' : '✗'} <strong>${d.name}</strong> (${d.topic}): ${d.status === 'ok' ? 'Responsive' : 'NO RESPONSE'}
                                    </li>
                                `).join('')}
                            </ul>
                        </div>

                    </div>
                ` : '<p class="secondary">No health check run recently.</p>'}
            </div>
        </section>
    `;

    app.innerHTML = `
        ${healthcheckHtml}
        <section class="admin-section">
            <header>
                <h3>Houses</h3>
            </header>
            <div id="admin-houses-list" class="admin-grid" aria-busy="true">Loading...</div>
        </section>

        <section class="admin-section">
            <h3>User Management</h3>
            <div id="admin-tenants" class="admin-grid" aria-busy="true">Loading...</div>
        </section>

        <section class="admin-section">
            <h3>Devices</h3>
            <div id="admin-devices" class="admin-grid" aria-busy="true">Loading...</div>
        </section>
    `;

    try {
        const [tenantsResp, devicesResp, housesResp, discoveryResp] = await Promise.all([
            fetch('/api/tenants'),
            fetch('/api/devices'),
            fetch('/api/houses'),
            fetch('/api/admin/discover-devices')
        ]);

        if (!tenantsResp.ok || !devicesResp.ok || !housesResp.ok) throw new Error('Failed to load admin data');

        const tenants = await tenantsResp.json();
        const devices = await devicesResp.json();
        const houses = await housesResp.json();
        const discovered = await discoveryResp.json();

        const houseMap = {};
        houses.forEach(h => { houseMap[h.id] = h.name; });

        // Houses list
        const housesDiv = document.getElementById('admin-houses-list');
        housesDiv.removeAttribute('aria-busy');
        houses.sort((a, b) => a.name.localeCompare(b.name));
        housesDiv.innerHTML = houses.map(h => renderHouseCard(h)).join('');

        const tenantMap = {};
        tenants.forEach(t => { tenantMap[t.id] = t.username; });

        // Tenants list
        const tenantsDiv = document.getElementById('admin-tenants');
        tenantsDiv.removeAttribute('aria-busy');
        tenants.sort((a, b) => a.username.localeCompare(b.username));
        tenantsDiv.innerHTML = tenants.map(t => renderTenantCard(t, houseMap[t.house_id])).join('');

        // Devices
        const devicesDiv = document.getElementById('admin-devices');
        devicesDiv.removeAttribute('aria-busy');
        devices.sort((a, b) => a.name.localeCompare(b.name));
        devicesDiv.innerHTML = devices.map(d => renderDeviceCard(d, tenantMap[d.tenant_id])).join('');

    } catch (e) {
        app.innerHTML += `<p style="color: red;">${e.message}</p>`;
    }
}

window.handleSchedulingChangeOverview = async (id, type) => {
    const untilContainer = document.getElementById(`ov-until-container-${id}`);
    const boilerConfig = document.getElementById(`ov-boiler-config-${id}`);
    
    if (type === 'FORCE_ON' || type === 'FORCE_OFF') {
        untilContainer.style.display = 'block';
        boilerConfig.style.display = 'none';
        const input = untilContainer.querySelector('input');
        if (!input.value) {
            const inOneHour = new Date(Date.now() + 3600000);
            const offset = inOneHour.getTimezoneOffset() * 60000;
            input.value = new Date(inOneHour.getTime() - offset).toISOString().slice(0, 16);
        }
    } else if (type === 'BOILER') {
        untilContainer.style.display = 'none';
        boilerConfig.style.display = 'block';
    } else {
        untilContainer.style.display = 'none';
        boilerConfig.style.display = 'none';
    }
};

window.updateDeviceConfigOverview = async (id) => {
    try {
        const type = document.getElementById(`ov-sch-${id}`).value;
        const until = document.getElementById(`ov-until-${id}`).value;
        
        const body = {
            expected_load: parseInt(document.getElementById(`ov-load-${id}`).value),
            full_charge_n_day: parseInt(document.getElementById(`ov-full-${id}`)?.value || 0),
            min_daily_charge: parseInt(document.getElementById(`ov-min-${id}`)?.value || 0),
            scheduling_type: {}
        };

        if (type === 'FORCE_ON' || type === 'FORCE_OFF') {
            body.scheduling_type = { type: type, until: new Date(until).toISOString() };
        } else {
            body.scheduling_type = { type: type };
        }

        const resp = await fetch(`/api/devices/${id}`, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body)
        });
        if (resp.ok) renderOverview();
        else alert('Update failed: ' + await resp.text());
    } catch (e) { alert('Error: ' + e); }
};

window.updateHouse = async (id, name, ha_host, ha_token) => {
    try {
        const params = new URLSearchParams({ name, ha_host, ha_token });
        const resp = await fetch(`/api/houses/${id}`, {
            method: 'PATCH',
            body: params
        });
        if (!resp.ok) alert('Update failed: ' + await resp.text());
    } catch (e) { alert('Update failed: ' + e); }
};

window.deleteHouse = async (id) => {
    if (!confirm('Are you sure you want to delete this house? This will also affect all tenants and devices associated with it.')) return;
    try {
        const resp = await fetch(`/api/houses/${id}`, { method: 'DELETE' });
        if (resp.ok) {
            closeModal();
            renderAdmin();
        }
        else alert('Failed to delete house: ' + await resp.text());
    } catch (err) { alert('Error: ' + err); }
};

window.deleteTenant = async (id) => {
    if (!confirm('Are you sure you want to delete this tenant? This cannot be undone.')) return;
    try {
        const resp = await fetch(`/api/tenants/${id}`, { method: 'DELETE' });
        if (resp.ok) {
            closeModal();
            renderAdmin();
        }
        else alert('Failed to delete: ' + await resp.text());
    } catch (err) { alert('Error: ' + err); }
}

window.deleteDevice = async (id) => {
    if (!confirm('Are you sure you want to delete this device? This will also remove its telemetry history.')) return;
    try {
        const resp = await fetch(`/api/devices/${id}`, { method: 'DELETE' });
        if (resp.ok) {
            closeModal();
            renderAdmin();
        }
        else alert('Failed to delete: ' + await resp.text());
    } catch (err) { alert('Error: ' + err); }
}

window.toggleDevice = async (id, context = 'overview') => {
    try {
        const resp = await fetch(`/api/devices/${id}/toggle`, { method: 'POST' });
        if (resp.ok) {
            if (context === 'admin') renderAdmin();
            else renderOverview();
        } else {
            const err = await resp.text();
            alert('Toggle failed: ' + err);
        }
    } catch (e) {
        alert('Toggle failed: ' + e);
    }
};

checkAuth();
