const app = document.getElementById('app');
const userInfo = document.getElementById('user-info');
const mainNav = document.getElementById('main-nav');
const adminTab = document.getElementById('admin-tab');

let currentUser = null;
let activeTab = 'overview';

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
    
    let selectorContainer = document.getElementById('house-selector-container');
    if (!selectorContainer) {
        // Recreate if it was cleared by renderLogin
        const li = document.createElement('li');
        li.id = 'house-selector-container';
        li.className = 'hidden';
        li.innerHTML = `<select id="house-selector" style="margin: 0; padding: 4px 8px; font-size: 0.8rem; height: auto; width: auto;"></select>`;
        userInfo.appendChild(li);
        selectorContainer = li;
    }

    if (currentUser.is_admin) {
        selectorContainer.classList.remove('hidden');
        populateHouseSelector();
    } else {
        selectorContainer.classList.add('hidden');
    }

    userInfo.innerHTML = ''; // Clear other items but we need to keep the selector
    userInfo.appendChild(selectorContainer);
    
    const userInfoList = document.createElement('li');
    userInfoList.innerHTML = `<span>${currentUser.username}</span>`;
    userInfo.appendChild(userInfoList);

    const logoutList = document.createElement('li');
    logoutList.innerHTML = `<button id="logout-btn" class="outline secondary" style="margin: 0; padding: 4px 12px; font-size: 0.8rem;">Logout</button>`;
    userInfo.appendChild(logoutList);

    // Add Change Password section
    if (!document.getElementById('change-password-container')) {
        const passContainer = document.createElement('div');
        passContainer.id = 'change-password-container';
        passContainer.style = 'position: fixed; bottom: 1rem; right: 1rem; z-index: 1000;';
        passContainer.innerHTML = `
            <details class="dropdown">
                <summary class="outline secondary" style="font-size: 0.8rem; padding: 4px 12px;">Settings</summary>
                <ul style="padding: 1rem; width: 250px; right: 0; left: auto;">
                    <li>
                        <form id="change-self-password-form" style="margin: 0;">
                            <label style="font-size: 0.8rem;">Change Password
                                <input type="password" name="password" placeholder="New Password" required style="font-size: 0.8rem; margin-bottom: 0.5rem;" />
                            </label>
                            <button type="submit" style="font-size: 0.8rem; padding: 4px 12px; margin: 0; width: 100%;">Update</button>
                        </form>
                    </li>
                </ul>
            </details>
        `;
        document.body.appendChild(passContainer);

        document.getElementById('change-self-password-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const formData = new FormData(e.target);
            const params = new URLSearchParams(formData);
            try {
                const resp = await fetch('/api/me/password', {
                    method: 'POST',
                    body: params
                });
                if (resp.ok) {
                    alert('Password updated successfully');
                    e.target.reset();
                    e.target.closest('details').open = false;
                } else {
                    alert('Update failed: ' + await resp.text());
                }
            } catch (err) { alert('Error: ' + err); }
        });
    }

    document.getElementById('logout-btn').addEventListener('click', async () => {
        await fetch('/api/logout', { method: 'POST' });
        currentUser = null;
        checkAuth();
    });

    if (currentUser.is_admin) {
        adminTab.closest('li').classList.remove('hidden');
    } else {
        adminTab.closest('li').classList.add('hidden');
        if (activeTab === 'admin') activeTab = 'overview';
    }

    renderActiveTab();
}

async function populateHouseSelector() {
    try {
        const resp = await fetch('/api/houses');
        if (!resp.ok) return;
        const houses = await resp.json();
        const selector = document.getElementById('house-selector');
        
        // Prevent re-adding listeners if already populated
        if (selector.dataset.populated) return;
        selector.dataset.populated = 'true';

        selector.innerHTML = '';
        houses.forEach(h => {
            const opt = document.createElement('option');
            opt.value = h.id;
            opt.textContent = h.name;
            if (h.id === currentUser.house_id) opt.selected = true;
            selector.appendChild(opt);
        });

        selector.addEventListener('change', async (e) => {
            await fetch('/api/admin/select-house', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ house_id: e.target.value })
            });
            window.location.reload();
        });
    } catch (e) {
        console.error('Failed to populate houses:', e);
    }
}

let overviewInterval = null;

function renderActiveTab() {
    if (overviewInterval) {
        clearInterval(overviewInterval);
        overviewInterval = null;
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
                    <span id="active-house-name" class="secondary" style="font-size: 0.9rem; font-weight: normal;"></span>
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

        const currentHouse = houses.find(h => h.id === currentUser.house_id);
        if (currentHouse) {
            const nameEl = document.getElementById('active-house-name');
            if (nameEl) nameEl.textContent = currentHouse.name;
        }

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
            let lastSeen = d.last_heartbeat ? new Date(d.last_heartbeat).toLocaleString() : 'Never';
            let statusText = `<code>${d.current_state}</code>`;
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

async function renderAdmin() {
    app.innerHTML = `
        <article>
            <header style="display: flex; justify-content: space-between; align-items: center;">
                <strong>Admin: Houses</strong>
                <span id="admin-house-name" class="secondary" style="font-size: 0.9rem; font-weight: normal;"></span>
            </header>
            <form id="create-house-form" style="margin-bottom: 2rem;">
                <div class="grid">
                    <label>House Name <input name="name" required autocomplete="off" /></label>
                    <label>HA Host <input name="ha_host" placeholder="192.168.1.100" required /></label>
                    <label>HA Token <input name="ha_token" required autocomplete="off" /></label>
                </div>
                <button type="submit" class="outline">Create House</button>
            </form>
            <div id="admin-houses-list" aria-busy="true">Loading...</div>
        </article>

        <article>
            <header><strong>User Management</strong></header>
            <form id="create-tenant-form" style="margin-bottom: 2rem;">
                <div class="grid">
                    <label>Username <input name="username" required autocomplete="off" /></label>
                    <label>Password <input name="password" type="password" placeholder="(default: username)" /></label>
                    <label>House 
                        <select name="house_id" id="admin-tenant-house-select" required></select>
                    </label>
                    <label style="display: flex; align-items: center; height: 100%; margin-top: 1rem;">
                        <input type="checkbox" name="is_admin" value="true" style="margin-right: 8px;"> Admin
                    </label>
                </div>
                <button type="submit" class="outline">Create User</button>
            </form>
            <div id="admin-tenants" aria-busy="true">Loading...</div>
        </article>

        <article>
            <header><strong>Add New Device</strong></header>
            <form id="create-device-form">
                <div class="grid">
                    <label>
                        Name
                        <input name="name" value="Boiler" required />
                    </label>
                    <label>
                        MQTT Topic
                        <input name="mqtt_topic" id="device-topic-input" placeholder="shellypro1-123456" required />
                        <div id="discovery-container" style="display: none; margin-top: 0.5rem;">
                            <small class="secondary">Discovered IDs: </small>
                            <span id="discovered-topics"></span>
                        </div>
                    </label>
                </div>
                <label>
                    Owner (Tenant)
                    <select name="tenant_id" id="tenant-select" required>
                        <option value="" disabled selected>Select a tenant...</option>
                    </select>
                </label>
                <button type="submit">Create Device</button>
            </form>
        </article>

        <article>
            <header><strong>Admin: All Devices in House</strong></header>
            <div id="admin-devices" aria-busy="true">Loading...</div>
        </article>
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

        const currentHouse = houses.find(h => h.id === currentUser.house_id);
        if (currentHouse) {
            document.getElementById('admin-house-name').textContent = currentHouse.name;
        }

        const houseMap = {};
        const houseSelect = document.getElementById('admin-tenant-house-select');
        houses.forEach(h => {
            houseMap[h.id] = h.name;
            const opt = document.createElement('option');
            opt.value = h.id;
            opt.textContent = h.name;
            if (h.id === currentUser.house_id) opt.selected = true;
            houseSelect.appendChild(opt);
        });

        // Houses list
        const housesDiv = document.getElementById('admin-houses-list');
        housesDiv.removeAttribute('aria-busy');
        let housesHtml = '<table><thead><tr><th>Name</th><th>HA Host</th><th>HA Token</th><th>Action</th></tr></thead><tbody>';
        for (const h of houses) {
            housesHtml += `
                <tr>
                    <td><input type="text" value="${h.name}" id="h-name-${h.id}" style="margin-bottom:0"></td>
                    <td><input type="text" value="${h.ha_host}" id="h-host-${h.id}" style="margin-bottom:0"></td>
                    <td><input type="text" value="${h.ha_token}" id="h-token-${h.id}" style="margin-bottom:0"></td>
                    <td>
                        <div class="grid" style="grid-template-columns: 1fr 1fr; gap: 8px;">
                            <button class="outline" style="margin:0; padding: 2px 8px;" onclick="updateHouse('${h.id}', document.getElementById('h-name-${h.id}').value, document.getElementById('h-host-${h.id}').value, document.getElementById('h-token-${h.id}').value)">Save</button>
                            <button class="outline secondary" style="margin:0; padding: 2px 8px;" onclick="deleteHouse('${h.id}')">Del</button>
                        </div>
                    </td>
                </tr>
            `;
        }
        housesHtml += '</tbody></table>';
        housesDiv.innerHTML = housesHtml;

        // Create House Form
        document.getElementById('create-house-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const formData = new FormData(e.target);
            const params = new URLSearchParams(formData);
            try {
                const resp = await fetch('/api/houses', {
                    method: 'POST',
                    body: params
                });
                if (resp.ok) renderAdmin();
                else alert('Failed to create house: ' + await resp.text());
            } catch (err) { alert('Error: ' + err); }
        });

        const tenantMap = {};
        const select = document.getElementById('tenant-select');
        tenants.forEach(t => {
            tenantMap[t.id] = t.username;
            const opt = document.createElement('option');
            opt.value = t.id;
            opt.textContent = t.username;
            select.appendChild(opt);
        });

        // Discovery
        if (discovered && discovered.length > 0) {
            const container = document.getElementById('discovery-container');
            const span = document.getElementById('discovered-topics');
            container.style.display = 'block';
            span.innerHTML = discovered.map(t => `<a href="#" onclick="setTopic('${t}'); return false;" style="margin-right: 8px; font-size: 0.8rem;">${t}</a>`).join('');
        }

        // Tenants
        const tenantsDiv = document.getElementById('admin-tenants');
        tenantsDiv.removeAttribute('aria-busy');
        let tenantsHtml = '<table><thead><tr><th>Username</th><th>House</th><th>Admin</th><th>New Password</th><th>Action</th></tr></thead><tbody>';
        for (const t of tenants) {
            const hOpts = houses.map(h => `<option value="${h.id}" ${h.id === t.house_id ? 'selected' : ''}>${h.name}</option>`).join('');
            tenantsHtml += `
                <tr>
                    <td><input type="text" value="${t.username}" id="t-user-${t.id}" style="margin-bottom:0"></td>
                    <td>
                        <select id="t-house-${t.id}" style="margin-bottom:0">
                            ${hOpts}
                        </select>
                    </td>
                    <td>
                        <input type="checkbox" id="t-admin-${t.id}" ${t.is_admin ? 'checked' : ''} style="margin-bottom:0">
                    </td>
                    <td>
                        <input type="password" id="t-pass-${t.id}" placeholder="Leave empty to keep" style="margin-bottom:0">
                    </td>
                    <td>
                        <div class="grid" style="grid-template-columns: 1fr 1fr; gap: 8px;">
                            <button class="outline" style="margin:0; padding: 2px 8px;" onclick="updateTenant('${t.id}')">Save</button>
                            <button class="outline secondary" style="margin:0; padding: 2px 8px;" onclick="deleteTenant('${t.id}')">Del</button>
                        </div>
                    </td>
                </tr>
            `;
        }
        tenantsHtml += '</tbody></table>';
        tenantsDiv.innerHTML = tenantsHtml;

        // Create Tenant Form
        document.getElementById('create-tenant-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const formData = new FormData(e.target);
            const params = new URLSearchParams(formData);
            try {
                const resp = await fetch('/api/tenants', {
                    method: 'POST',
                    body: params
                });
                if (resp.ok) renderAdmin();
                else alert('Failed to create tenant: ' + await resp.text());
            } catch (err) { alert('Error: ' + err); }
        });

        // Create Device Form
        document.getElementById('create-device-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const formData = new FormData(e.target);
            const params = new URLSearchParams(formData);
            try {
                const resp = await fetch('/api/devices', {
                    method: 'POST',
                    body: params
                });
                if (resp.ok) renderAdmin();
                else alert('Failed to create device: ' + await resp.text());
            } catch (err) { alert('Error: ' + err); }
        });

        // Devices
        const devicesDiv = document.getElementById('admin-devices');
        devicesDiv.removeAttribute('aria-busy');
        let devicesHtml = '<table><thead><tr><th>Device Info</th><th>Owner</th><th>Config</th><th>Scheduling</th><th>Action</th></tr></thead><tbody>';
        for (const d of devices) {
            const tOpts = tenants.map(t => `<option value="${t.id}" ${t.id === d.tenant_id ? 'selected' : ''}>${t.username}</option>`).join('');
            const schObj = d.scheduling_type;
            const schType = (schObj && typeof schObj === 'object' ? schObj.type : schObj) || 'UNKNOWN';
            
            let until = '';
            if (schObj && schObj.until) {
                const date = new Date(schObj.until);
                const offset = date.getTimezoneOffset() * 60000;
                until = new Date(date.getTime() - offset).toISOString().slice(0, 16);
            }

            devicesHtml += `
                <tr>
                    <td data-label="Name">
                        <input type="text" value="${d.name}" id="d-name-${d.id}" placeholder="Name" style="margin-bottom:0.5rem">
                        <input type="text" value="${d.mqtt_topic}" id="d-topic-${d.id}" placeholder="Topic" style="margin-bottom:0">
                    </td>
                    <td data-label="Owner">
                        <select id="d-owner-${d.id}" style="margin-bottom:0">
                            ${tOpts}
                        </select>
                    </td>
                    <td data-label="Config">
                        <label>Load (W)
                            <input type="number" value="${d.expected_load}" id="d-load-${d.id}" style="margin-bottom:0.5rem">
                        </label>
                        <div id="boiler-config-${d.id}" style="${schType === 'BOILER' ? '' : 'display:none'}">
                            <label>Charge (Days)
                                <input type="number" min="1" max="8" value="${d.full_charge_n_day}" id="d-full-${d.id}" style="margin-bottom:0.5rem">
                            </label>
                            <label>Min (Mins)
                                <input type="number" min="0" value="${d.min_daily_charge}" id="d-min-${d.id}" style="margin-bottom:0">
                            </label>
                        </div>
                    </td>
                    <td data-label="Scheduling">
                        <label>Mode
                            <select id="d-sch-${d.id}" onchange="handleSchedulingChange('${d.id}', this.value, '${d.mqtt_topic}')" style="margin-bottom:0.5rem">
                                <option value="BOILER" ${schType === 'BOILER' ? 'selected' : ''}>Boiler</option>
                                <option value="NONE" ${schType === 'NONE' ? 'selected' : ''}>Manual</option>
                                <option value="FORCE_ON" ${schType === 'FORCE_ON' ? 'selected' : ''}>Force ON</option>
                                <option value="FORCE_OFF" ${schType === 'FORCE_OFF' ? 'selected' : ''}>Force OFF</option>
                            </select>
                        </label>
                        <div id="until-container-${d.id}" style="${(schType === 'FORCE_ON' || schType === 'FORCE_OFF') ? '' : 'display:none'}">
                            <label>Until
                                <input type="datetime-local" value="${until}" id="d-until-${d.id}" style="margin-bottom:0">
                            </label>
                        </div>
                    </td>
                    <td data-label="Action">
                        <button class="outline" style="margin-bottom:0.5rem; padding: 2px 8px; width: 100%" onclick="updateDeviceConfigAdmin('${d.id}')">Save All</button>
                        <button class="outline secondary" style="margin:0; padding: 2px 8px; width: 100%" onclick="deleteDevice('${d.id}')">Delete</button>
                    </td>
                </tr>
            `;
        }
        devicesHtml += '</tbody></table>';
        devicesDiv.innerHTML = devicesHtml;

    } catch (e) {
        app.innerHTML += `<p style="color: red;">${e.message}</p>`;
    }
}

window.setTopic = (topic) => {
    document.getElementById('device-topic-input').value = topic;
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

window.deleteHouse = async (id) => {
    if (!confirm('Are you sure you want to delete this house? This will also affect all tenants and devices associated with it.')) return;
    try {
        const resp = await fetch(`/api/houses/${id}`, { method: 'DELETE' });
        if (resp.ok) renderAdmin();
        else alert('Failed to delete house: ' + await resp.text());
    } catch (err) { alert('Error: ' + err); }
};

window.deleteTenant = async (id) => {
    if (!confirm('Are you sure you want to delete this tenant? This cannot be undone.')) return;
    try {
        const resp = await fetch(`/api/tenants/${id}`, { method: 'DELETE' });
        if (resp.ok) renderAdmin();
        else alert('Failed to delete: ' + await resp.text());
    } catch (err) { alert('Error: ' + err); }
}

window.deleteDevice = async (id) => {
    if (!confirm('Are you sure you want to delete this device? This will also remove its telemetry history.')) return;
    try {
        const resp = await fetch(`/api/devices/${id}`, { method: 'DELETE' });
        if (resp.ok) renderAdmin();
        else alert('Failed to delete: ' + await resp.text());
    } catch (err) { alert('Error: ' + err); }
}

window.updateHouse = async (id, name, ha_host, ha_token) => {
    try {
        const params = new URLSearchParams({ name, ha_host, ha_token });
        const resp = await fetch(`/api/houses/${id}`, {
            method: 'PATCH',
            body: params
        });
        if (resp.ok) renderAdmin();
        else alert('Update failed: ' + await resp.text());
    } catch (e) { alert('Update failed: ' + e); }
};

window.updateTenant = async (id) => {
    try {
        const username = document.getElementById(`t-user-${id}`).value;
        const house_id = document.getElementById(`t-house-${id}`).value;
        const is_admin = document.getElementById(`t-admin-${id}`).checked;
        const password = document.getElementById(`t-pass-${id}`).value;
        
        const params = new URLSearchParams({ username, house_id, is_admin });
        if (password) params.append('password', password);
        
        const resp = await fetch(`/api/tenants/${id}`, {
            method: 'PATCH',
            body: params
        });
        if (resp.ok) renderAdmin();
        else alert('Update failed: ' + await resp.text());
    } catch (e) { alert('Update failed: ' + e); }
};

window.handleSchedulingChange = async (id, type, mqtt_topic) => {
    const untilContainer = document.getElementById(`until-container-${id}`);
    const boilerConfig = document.getElementById(`boiler-config-${id}`);
    
    if (type === 'FORCE_ON' || type === 'FORCE_OFF') {
        untilContainer.style.display = 'block';
        boilerConfig.style.display = 'none';
        // Default until to 1 hour from now if not set
        const input = untilContainer.querySelector('input');
        if (!input.value) {
            const inOneHour = new Date(Date.now() + 3600000);
            const offset = inOneHour.getTimezoneOffset() * 60000;
            input.value = new Date(inOneHour.getTime() - offset).toISOString().slice(0, 16);
        }
        updateDeviceScheduling(id, type, input.value);
    } else if (type === 'BOILER') {
        untilContainer.style.display = 'none';
        boilerConfig.style.display = 'block';
        updateDeviceScheduling(id, type, null);
    } else {
        untilContainer.style.display = 'none';
        boilerConfig.style.display = 'none';
        updateDeviceScheduling(id, type, null);
    }
};

window.updateDeviceConfig = async (id, load, full_charge, min_charge) => {
    try {
        const body = {};
        if (load !== null) body.expected_load = parseInt(load);
        if (full_charge !== null) body.full_charge_n_day = parseInt(full_charge);
        if (min_charge !== null) body.min_daily_charge = parseInt(min_charge);

        const resp = await fetch(`/api/devices/${id}`, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body)
        });
        if (!resp.ok) alert('Update failed');
    } catch (e) {
        alert('Update failed: ' + e);
    }
};

window.updateDeviceConfigAdmin = async (id) => {
    try {
        const body = {
            name: document.getElementById(`d-name-${id}`).value,
            mqtt_topic: document.getElementById(`d-topic-${id}`).value,
            tenant_id: document.getElementById(`d-owner-${id}`).value,
            expected_load: parseInt(document.getElementById(`d-load-${id}`).value),
            full_charge_n_day: parseInt(document.getElementById(`d-full-${id}`)?.value || 0),
            min_daily_charge: parseInt(document.getElementById(`d-min-${id}`)?.value || 0),
        };

        const resp = await fetch(`/api/devices/${id}`, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body)
        });
        if (resp.ok) renderAdmin();
        else alert('Update failed: ' + await resp.text());
    } catch (e) { alert('Error: ' + e); }
};

window.updateDeviceScheduling = async (id, type, until) => {
    try {
        const body = { scheduling_type: {} };
        if (!type) {
            // Finding the current type from the select
            const row = document.querySelector(`select[onchange*="${id}"]`);
            type = row.value;
        }

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
        if (!resp.ok) alert('Update failed');
    } catch (e) {
        alert('Update failed: ' + e);
    }
};

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
