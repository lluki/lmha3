const app = document.getElementById('app');
const userInfo = document.getElementById('user-info');
const mainNav = document.getElementById('main-nav');
const adminTab = document.getElementById('admin-tab');
const adminModal = document.getElementById('admin-modal');
const modalTitle = document.getElementById('modal-title');
const modalContent = document.getElementById('modal-content');
const createBtn = document.getElementById('floating-create-btn');

const formatTime = (ts) => {
    if (!ts) return 'Never';
    return new Date(ts).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
};

const formatDateTime = (ts) => {
    if (!ts) return 'Never';
    return new Date(ts).toLocaleString(undefined, { 
        year: 'numeric', month: '2-digit', day: '2-digit', 
        hour: '2-digit', minute: '2-digit' 
    });
};

let currentUser = null;
let activeTab = 'overview';
let healthcheckLoading = false;
let healthcheckResults = null;
let currentDeviceId = null;
let currentDeviceTab = 'settings';

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
                    <strong>${currentUser.is_admin ? 'All Devices' : 'My Devices'}</strong>
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

        const houseMap = {};
        houses.forEach(h => houseMap[h.id] = h);

        const userDevices = currentUser.is_admin ? devices : devices.filter(d => d.tenant_id === currentUser.tenant_id);
        const content = document.getElementById('overview-content');
        if (!content) return;
        content.removeAttribute('aria-busy');

        if (userDevices.length === 0) {
            content.innerHTML = '<p>No devices found.</p>';
            return;
        }

        let html = '<div class="grid">';

        for (const d of userDevices) {
            const house = houseMap[d.house_id];
            // Offline detection (5 minutes threshold)
            const lastFeedback = d.last_feedback_time ? new Date(d.last_feedback_time).getTime() : 0;
            const isHealthy = (Date.now() - lastFeedback) < (5 * 60 * 1000);
            const healthStatus = isHealthy ? "Yes" : `No, ${formatDateTime(d.last_feedback_time)}`;
            
            // Mode detection
            const schObj = d.scheduling_type;
            const schType = (schObj && typeof schObj === 'object' ? schObj.type : schObj) || 'UNKNOWN';
            let modeText = "Normal";
            let isVacationActive = false;
            if (schType === 'FORCE_OFF' && schObj.until) {
                const returnDate = new Date(new Date(schObj.until).getTime() + 24*60*60*1000);
                modeText = `Vacation mode until ${returnDate.toLocaleDateString('de-CH')}`;
                isVacationActive = true;
            }

            // Runtime
            const hours = Math.floor(d.runtime_24h / 60);
            const mins = d.runtime_24h % 60;
            const runtimeText = `${hours}h ${mins}m`;

            html += `
                <article>
                    <header>
                        <strong>${d.name}</strong>
                        ${currentUser.is_admin ? `<br><small class="secondary" style="font-size: 0.7rem;">Owner: ${tenantMap[d.tenant_id] || 'Unknown'}</small>` : ''}
                    </header>
                    <div style="font-size: 0.9rem;">
                        <p style="margin-bottom: 0.5rem;"><strong>Healthy:</strong> ${healthStatus}</p>
                        <p style="margin-bottom: 0.5rem;"><strong>Status:</strong> ${d.current_state}</p>
                        <p style="margin-bottom: 0.5rem;"><strong>Today's Runtime:</strong> ${runtimeText}</p>
                        <p style="margin-bottom: 1rem;"><strong>Mode:</strong> ${modeText}</p>
                    </div>
                    <footer>
                        <button class="primary" style="width: 100%; padding: 0.8rem;" onclick="openVacationModal('${d.id}', ${isVacationActive}, '${house ? house.day_deadline : ''}')">
                            ${isVacationActive ? 'Modify Vacation Absence' : 'Set Vacation Absence'}
                        </button>
                    </footer>
                </article>
            `;
        }
        html += '</div>';
        content.innerHTML = html;
    } catch (e) {
        const content = document.getElementById('overview-content');
        if (content) {
            content.removeAttribute('aria-busy');
            content.innerHTML = `<p style="color: red;">${e.message}</p>`;
        }
    }
}

const vacationModal = document.getElementById('vacation-modal');
const cancelVacationBtn = document.getElementById('cancel-vacation-btn');

window.openVacationModal = (deviceId, isVacationActive, deadline) => {
    document.getElementById('vacation-device-id').value = deviceId;
    document.getElementById('vacation-return-date').value = new Date().toISOString().split('T')[0];
    cancelVacationBtn.style.display = isVacationActive ? 'block' : 'none';
    
    if (deadline) {
        document.getElementById('vacation-deadline-note').textContent = `The boiler will resume heating after ${deadline.slice(0, 5)} on the day prior to your return.`;
    }

    vacationModal.showModal();
};

window.closeVacationModal = () => {
    vacationModal.close();
};

window.cancelVacation = async () => {
    const id = document.getElementById('vacation-device-id').value;
    if (!confirm('Are you sure you want to cancel the vacation absence and return to Normal mode?')) return;

    try {
        const resp = await fetch(`/api/devices/${id}`, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ scheduling_type: { type: 'BOILER' } })
        });
        
        if (resp.ok) {
            closeVacationModal();
            renderOverview();
        } else {
            alert('Failed to cancel vacation: ' + await resp.text());
        }
    } catch (e) {
        alert('Error cancelling vacation: ' + e);
    }
};

window.saveVacation = async () => {
    const id = document.getElementById('vacation-device-id').value;
    const returnDate = document.getElementById('vacation-return-date').value;
    
    try {
        const resp = await fetch(`/api/devices/${id}/vacation`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ return_date: returnDate })
        });
        
        if (resp.ok) {
            closeVacationModal();
            renderOverview();
        } else {
            alert('Failed to set vacation: ' + await resp.text());
        }
    } catch (e) {
        alert('Error setting vacation: ' + e);
    }
};

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
                    <td>${formatDateTime(t.timestamp)}</td>
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
                    [${formatTime(log.timestamp)}] <strong>${log.level}</strong> [${log.target}] ${log.message}
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
    const lastFeedback = d.last_feedback_time ? new Date(d.last_feedback_time).getTime() : 0;
    const lastRequest = d.last_request_time ? new Date(d.last_request_time).getTime() : 0;
    const isOffline = (lastRequest > lastFeedback) && (lastRequest > lastFeedback + 20000);
    const isSyncing = d.desired_state !== d.current_state;

    return `
        <article class="summary-card" onclick="openDeviceDetails('${d.id}')">
            <header>
                <div style="display: flex; justify-content: space-between; align-items: flex-start; width: 100%;">
                    <div>
                        <strong>${d.name}</strong><br>
                        <span class="secondary" style="font-size: 0.8rem;">${schType} ${isSyncing ? '<small style="color: var(--pico-ins-color)">(Syncing...)</small>' : ''}</span>
                    </div>
                    <div style="display: flex; align-items: center; gap: 0.5rem;">
                        <button class="outline" style="padding: 2px 8px; font-size: 0.7rem; margin: 0; white-space: nowrap;" onclick="event.stopPropagation(); window.toggleDevice('${d.id}', 'admin')">Toggle</button>
                        ${isOffline ? '<span style="background: var(--pico-del-color); color: white; padding: 1px 4px; border-radius: 3px; font-size: 0.6rem; font-weight: bold;">OFFLINE</span>' : ''}
                    </div>
                </div>
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

async function renderDeviceDetails(id, isEdit = false, tab = 'settings') {
    try {
        const [tenantsResp, devicesResp] = await Promise.all([
            fetch('/api/tenants'),
            fetch('/api/devices')
        ]);
        const tenants = await tenantsResp.json();
        const devices = await devicesResp.json();
        const d = devices.find(x => x.id === id);
        if (!d) return;

        currentDeviceId = id;
        currentDeviceTab = tab;

        const tenantName = tenants.find(t => t.id === d.tenant_id)?.username || 'Unknown';
        const schObj = d.scheduling_type;
        const schType = (schObj && typeof schObj === 'object' ? schObj.type : schObj) || 'UNKNOWN';
        
        let until = '';
        if (schObj && schObj.until) {
            const date = new Date(schObj.until);
            const offset = date.getTimezoneOffset() * 60000;
            until = new Date(date.getTime() - offset).toISOString().slice(0, 16);
        }

        let header = `
            <nav>
                <ul style="margin-bottom: 1.5rem;">
                    <li><button class="${tab === 'settings' ? '' : 'outline'} secondary" style="margin: 0;" onclick="openDeviceDetails('${id}', 'settings')">Settings</button></li>
                    <li><button class="${tab === 'scripts' ? '' : 'outline'} secondary" style="margin: 0;" onclick="openDeviceDetails('${id}', 'scripts')">Scripts</button></li>
                </ul>
            </nav>
            <div id="device-tab-content">
        `;

        let footer = `</div>`;
        let content = '';

        if (tab === 'settings') {
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
                        <div id="modal-boiler-config">
                            <label>Runtime (Minutes) <input type="number" name="device_runtime" min="0" value="${d.device_runtime}" /></label>
                        </div>
                        <div id="modal-until-container" style="${(schType === 'FORCE_ON' || schType === 'FORCE_OFF') ? '' : 'display:none'}">
                            <label>Until <input type="datetime-local" name="scheduling_until" value="${until}" /></label>
                        </div>
                        <div class="grid" style="margin-top: 2rem;">
                            <button type="submit">Save</button>
                            <button type="button" class="secondary" onclick="renderDeviceDetails('${d.id}', false, 'settings')">Cancel</button>
                        </div>
                    </form>
                `;
            } else {
                const lastFeedback = formatDateTime(d.last_feedback_time);
                const lastRequest = formatDateTime(d.last_request_time);
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
                    <div class="detail-row"><span class="detail-label">Last Heartbeat</span><span>${lastFeedback}</span></div>
                    ${(until && (schType === 'FORCE_ON' || schType === 'FORCE_OFF')) ? `<div class="detail-row"><span class="detail-label">Mode Until</span><span>${formatDateTime(schObj.until)}</span></div>` : ''}
                    <div class="detail-row"><span class="detail-label">Daily Runtime</span><span>${d.device_runtime} mins</span></div>
                    <div class="grid" style="margin-top: 2rem;">
                        <button onclick="window.toggleDevice('${d.id}', 'admin')">Toggle Device</button>
                        <button class="outline" onclick="renderDeviceDetails('${d.id}', true, 'settings')">Edit Config</button>
                        <button class="secondary outline" onclick="deleteDevice('${d.id}')">Delete Device</button>
                    </div>
                `;
            }
        } else if (tab === 'scripts') {
            content = `<p aria-busy="true">Loading scripts...</p>`;
        }

        showModal(`Device: ${d.name}`, header + content + footer);

        if (tab === 'scripts') {
            renderDeviceScriptsTab(d);
        }

        if (isEdit && tab === 'settings') {
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
                    device_runtime: parseInt(formData.get('device_runtime') || 0),
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

async function renderDeviceScriptsTab(device) {
    const content = document.getElementById('device-tab-content');
    if (!content) return;
    
    try {
        const resp = await fetch(`/api/admin/devices/${device.id}/scripts`);
        if (!resp.ok) throw new Error(await resp.text());
        const data = await resp.json();
        const scripts = data.result?.scripts || [];
        
        if (scripts.length === 0) {
            content.innerHTML = '<p>No scripts found on this device.</p>';
            return;
        }

        let html = '<table><thead><tr><th>Name</th><th>Status</th><th>Actions</th></tr></thead><tbody>';
        scripts.forEach(s => {
            html += `
                <tr>
                    <td>${s.name}</td>
                    <td>${s.running ? '<mark style="background: var(--pico-ins-color); color: white; padding: 2px 6px; border-radius: 4px;">Running</mark>' : 'Stopped'}</td>
                    <td style="display: flex; gap: 0.5rem;">
                        ${s.running ? 
                            `<button class="outline contrast" style="padding: 2px 8px; font-size: 0.8rem; margin: 0;" onclick="handleScriptAction('${device.id}', ${s.id}, 'stop')">Stop</button>` : 
                            `<button class="outline" style="padding: 2px 8px; font-size: 0.8rem; margin: 0;" onclick="handleScriptAction('${device.id}', ${s.id}, 'start')">Start</button>`
                        }
                        <button class="outline secondary" style="padding: 2px 8px; font-size: 0.8rem; margin: 0;" onclick="renderScriptEditor('${device.id}', ${s.id}, '${s.name}')">Edit</button>
                    </td>
                </tr>
            `;
        });
        html += '</tbody></table>';
        content.innerHTML = html;
    } catch (e) {
        content.innerHTML = `<p style="color: red;">Error: ${e.message}</p>`;
    }
}

async function renderScriptEditor(deviceId, scriptId, scriptName) {
    const content = document.getElementById('device-tab-content');
    if (!content) return;

    content.innerHTML = `<p aria-busy="true">Fetching script code...</p>`;

    try {
        const resp = await fetch(`/api/admin/devices/${deviceId}/scripts/${scriptId}/code`);
        if (!resp.ok) throw new Error(await resp.text());
        const data = await resp.json();
        console.log('Script.GetCode response:', data);
        
        // According to logs, Shelly returns code in 'result.data'
        const code = (data.result?.data ?? data.result?.code ?? data.code) || '';

        content.innerHTML = `
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
                <h5 style="margin: 0;">Editing: ${scriptName}</h5>
                <button class="outline secondary" style="padding: 2px 8px; font-size: 0.8rem; margin: 0;" onclick="openDeviceDetails('${deviceId}', 'scripts')">Back to List</button>
            </div>
            <textarea id="script-code-editor" style="font-family: monospace; height: 400px; margin-bottom: 1rem;"></textarea>
            <div style="display: flex; gap: 1rem;">
                <button id="save-script-btn" onclick="saveScriptCode('${deviceId}', ${scriptId})">Save Changes</button>
            </div>
        `;

        // Set value via property to avoid HTML escaping issues in innerHTML
        document.getElementById('script-code-editor').value = code;
    } catch (e) {
        console.error('Error fetching script code:', e);
        content.innerHTML = `<p style="color: red;">Error fetching code: ${e.message}</p>
        <button class="outline secondary" onclick="openDeviceDetails('${deviceId}', 'scripts')">Back</button>`;
    }
}

window.saveScriptCode = async (deviceId, scriptId) => {
    const btn = document.getElementById('save-script-btn');
    const code = document.getElementById('script-code-editor').value;
    
    btn.setAttribute('aria-busy', 'true');
    btn.disabled = true;

    try {
        const resp = await fetch(`/api/admin/devices/${deviceId}/scripts/${scriptId}/code`, {
            method: 'PUT',
            body: code
        });
        
        if (resp.ok) {
            alert('Script saved successfully!');
            openDeviceDetails(deviceId, 'scripts');
        } else {
            alert('Save failed: ' + await resp.text());
        }
    } catch (e) {
        alert('Error: ' + e);
    } finally {
        btn.removeAttribute('aria-busy');
        btn.disabled = false;
    }
};

window.renderScriptEditor = renderScriptEditor;

window.handleScriptAction = async (deviceId, scriptId, action) => {
    try {
        const resp = await fetch(`/api/admin/devices/${deviceId}/scripts/${scriptId}/${action}`, { method: 'POST' });
        if (resp.ok) {
            const devicesResp = await fetch('/api/devices');
            const devices = await devicesResp.json();
            const d = devices.find(x => x.id === deviceId);
            renderDeviceScriptsTab(d);
        } else {
            alert('Action failed: ' + await resp.text());
        }
    } catch (e) {
        alert('Error: ' + e);
    }
};

window.openDeviceDetails = (id, tab = 'settings') => renderDeviceDetails(id, false, tab);

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
            <label>HA URL <input name="ha_url" placeholder="http://192.168.1.100:8123" required /></label>
            <label>HA Token <input name="ha_token" required autocomplete="off" /></label>
            <div class="grid">
                <label>PV Entity ID <input name="ha_pv_entity_id" value="sensor.panel_production_power" required /></label>
                <label>Consumption Entity ID <input name="ha_consumption_entity_id" value="sensor.house_load_power" required /></label>
            </div>
            <label>Day Deadline (HH:MM) <input name="day_deadline" value="05:00" type="time" required /></label>
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
    
    if (type === 'FORCE_ON' || type === 'FORCE_OFF') {
        untilContainer.style.display = 'block';
        const input = untilContainer.querySelector('input');
        if (!input.value) {
            const inOneHour = new Date(Date.now() + 3600000);
            const offset = inOneHour.getTimezoneOffset() * 60000;
            input.value = new Date(inOneHour.getTime() - offset).toISOString().slice(0, 16);
        }
    } else {
        untilContainer.style.display = 'none';
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
                <div>URL: ${h.ha_url}</div>
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
                    <label>HA URL <input name="ha_url" value="${h.ha_url}" required /></label>
                    <label>HA Token <input name="ha_token" value="${h.ha_token}" required /></label>
                    <div class="grid">
                        <label>PV Entity ID <input name="ha_pv_entity_id" value="${h.ha_pv_entity_id}" required /></label>
                        <label>Consumption Entity ID <input name="ha_consumption_entity_id" value="${h.ha_consumption_entity_id}" required /></label>
                    </div>
                    <label>Day Deadline (HH:MM) <input name="day_deadline" value="${h.day_deadline.slice(0, 5)}" type="time" required /></label>
                    <div class="grid">
                        <button type="submit">Save Changes</button>
                        <button type="button" class="secondary" onclick="renderHouseDetails('${h.id}', false)">Cancel</button>
                    </div>
                </form>
            `;
        } else {
            content = `
                <div class="detail-row"><span class="detail-label">Name</span><span>${h.name}</span></div>
                <div class="detail-row"><span class="detail-label">HA URL</span><span>${h.ha_url}</span></div>
                <div class="detail-row"><span class="detail-label">HA Token</span><span>••••••••</span></div>
                <div class="detail-row"><span class="detail-label">PV Entity</span><span>${h.ha_pv_entity_id}</span></div>
                <div class="detail-row"><span class="detail-label">Consumption Entity</span><span>${h.ha_consumption_entity_id}</span></div>
                <div class="detail-row"><span class="detail-label">Day Deadline</span><span>${h.day_deadline.slice(0, 5)}</span></div>
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
                await updateHouse(h.id, formData.get('name'), formData.get('ha_url'), formData.get('ha_token'), formData.get('ha_pv_entity_id'), formData.get('ha_consumption_entity_id'), formData.get('day_deadline'));
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

window.updateHouse = async (id, name, ha_url, ha_token, ha_pv_entity_id, ha_consumption_entity_id, day_deadline) => {
    try {
        const params = new URLSearchParams({ name, ha_url, ha_token, ha_pv_entity_id, ha_consumption_entity_id, day_deadline });
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
