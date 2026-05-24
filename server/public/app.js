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
    userInfo.innerHTML = `
        <li><span>${currentUser.username}</span></li>
        <li><button id="logout-btn" class="outline secondary" style="margin: 0; padding: 4px 12px; font-size: 0.8rem;">Logout</button></li>
    `;

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

function renderActiveTab() {
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
            break;
        case 'history':
            renderHistory();
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
    app.innerHTML = `
        <article>
            <header><strong>${currentUser.is_admin ? 'All Devices' : 'Your Devices'}</strong></header>
            <div id="overview-content" aria-busy="true">Loading...</div>
        </article>
    `;
    
    try {
        const fetchTasks = [
            fetch('/api/devices'),
            fetch('/api/history')
        ];
        if (currentUser.is_admin) {
            fetchTasks.push(fetch('/api/tenants'));
        }

        const responses = await Promise.all(fetchTasks);
        for (const resp of responses) {
            if (!resp.ok) throw new Error('Failed to load data');
        }
        
        const devices = await responses[0].json();
        const history = await responses[1].json();
        const tenants = currentUser.is_admin ? await responses[2].json() : [];
        const tenantMap = {};
        tenants.forEach(t => tenantMap[t.id] = t.username);
        
        const userDevices = currentUser.is_admin ? devices : devices.filter(d => d.tenant_id === currentUser.tenant_id);
        const content = document.getElementById('overview-content');
        content.removeAttribute('aria-busy');

        if (userDevices.length === 0) {
            content.innerHTML = '<p>No devices found.</p>';
            return;
        }

        let html = '<table><thead><tr><th>Name</th>';
        if (currentUser.is_admin) html += '<th>Owner</th>';
        html += '<th>Status</th><th>Last Seen</th><th>Action</th></tr></thead><tbody>';

        for (const d of userDevices) {
            let lastSeen = d.last_heartbeat ? new Date(d.last_heartbeat).toLocaleString() : 'Never';
            let statusText = `<code>${d.current_state}</code>`;
            
            if (d.current_state === 'ON') {
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
                    <td data-label="Status">${statusText}</td>
                    <td data-label="Last Seen">${lastSeen}</td>
                    <td data-label="Action">
                        <button class="outline" style="margin:0; padding: 2px 8px;" onclick="toggleDevice('${d.id}')">Toggle</button>
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
                        <input type="checkbox" id="show-consumption" style="margin-right: 8px;" />
                        <span>Show Consumption</span>
                    </label>
                </fieldset>
            </header>
            <div id="history-content" aria-busy="true">Loading...</div>
        </article>
    `;
    
    const showConsumption = document.getElementById('show-consumption');
    showConsumption.addEventListener('change', () => fetchAndRenderHistory(showConsumption.checked));
    
    fetchAndRenderHistory(false);
}

async function fetchAndRenderHistory(includeConsumption) {
    const content = document.getElementById('history-content');
    content.setAttribute('aria-busy', 'true');
    
    try {
        const [historyResp, devicesResp, tenantsResp] = await Promise.all([
            fetch('/api/history'),
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
        if (!includeConsumption) {
            filtered = history.filter(t => t.source === 'DEVICE_STATE');
        }
        
        if (filtered.length === 0) {
            content.innerHTML = '<p>No history found.</p>';
            return;
        }

        let html = '<table><thead><tr><th>Time</th><th>Device</th><th>Event</th><th>Value</th></tr></thead><tbody>';
        for (const t of filtered) {
            const device = devices.find(d => d.id === t.device_id);
            let deviceName = 'System';
            if (device) {
                const ownerName = tenantMap[device.tenant_id] || 'unknown';
                deviceName = `${device.name} <small class="secondary">(${ownerName})</small>`;
            }
            let eventType = t.source;
            let valText = t.value;
            
            if (t.source === 'DEVICE_STATE') {
                eventType = 'State';
                valText = t.value === 1.0 ? '<mark style="background: var(--pico-ins-color); color: white; padding: 2px 6px; border-radius: 4px;">ON</mark>' : '<mark style="background: var(--pico-del-color); color: white; padding: 2px 6px; border-radius: 4px;">OFF</mark>';
            } else if (t.source === 'DEVICE_CONSUMPTION') {
                eventType = 'Consumption';
                valText = `<code>${t.value.toFixed(1)} W</code>`;
            } else if (t.source === 'PV_PRODUCTION') {
                eventType = 'PV Production';
                valText = `<code>${t.value.toFixed(1)} W</code>`;
            } else if (t.source === 'HOUSE_CONSUMPTION') {
                eventType = 'House Consumption';
                valText = `<code>${t.value.toFixed(1)} W</code>`;
            }
            
            html += `
                <tr>
                    <td>${new Date(t.timestamp).toLocaleTimeString()} <small class="secondary">${new Date(t.timestamp).toLocaleDateString()}</small></td>
                    <td>${deviceName}</td>
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

async function renderAdmin() {
    app.innerHTML = `
        <article>
            <header><strong>Admin: Tenants</strong></header>
            <div id="admin-tenants" aria-busy="true">Loading...</div>
        </article>

        <article>
            <header><strong>Add New Device</strong></header>
            <form id="create-device-form">
                <div class="grid">
                    <label>
                        Name
                        <input name="name" value="Boiler" placeholder="Living Room Light" required />
                    </label>
                    <label>
                        MQTT Topic
                        <input name="mqtt_topic" placeholder="shellypro1-123456" required />
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
            <header><strong>Admin: All Devices</strong></header>
            <div id="admin-devices" aria-busy="true">Loading...</div>
        </article>
    `;

    try {
        const [tenantsResp, devicesResp] = await Promise.all([
            fetch('/api/tenants'),
            fetch('/api/devices')
        ]);

        if (!tenantsResp.ok || !devicesResp.ok) throw new Error('Failed to load admin data');

        const tenants = await tenantsResp.json();
        const devices = await devicesResp.json();

        const tenantMap = {};
        const select = document.getElementById('tenant-select');
        tenants.forEach(t => {
            tenantMap[t.id] = t.username;
            const opt = document.createElement('option');
            opt.value = t.id;
            opt.textContent = t.username;
            select.appendChild(opt);
        });

        // Tenants
        const tenantsDiv = document.getElementById('admin-tenants');
        tenantsDiv.removeAttribute('aria-busy');
        let tenantsHtml = '<ul>';
        for (const t of tenants) {
            tenantsHtml += `<li><strong>${t.username}</strong> <small class="secondary">(${t.id})</small></li>`;
        }
        tenantsHtml += '</ul>';
        tenantsDiv.innerHTML = tenantsHtml;

        // Form logic
        document.getElementById('create-device-form').addEventListener('submit', async (e) => {
            e.preventDefault();
            const formData = new FormData(e.target);
            const params = new URLSearchParams(formData);
            try {
                const resp = await fetch('/api/devices', {
                    method: 'POST',
                    body: params
                });
                if (resp.ok) {
                    renderAdmin();
                } else {
                    const err = await resp.text();
                    alert('Failed to create device: ' + err);
                }
            } catch (err) {
                alert('Failed to create device: ' + err);
            }
        });

        // Devices
        const devicesDiv = document.getElementById('admin-devices');
        devicesDiv.removeAttribute('aria-busy');
        let devicesHtml = '<table><thead><tr><th>Name</th><th>Owner</th><th>Topic</th><th>Status</th><th>Action</th></tr></thead><tbody>';
        for (const d of devices) {
            const ownerName = tenantMap[d.tenant_id] || 'Unknown';
            devicesHtml += `
                <tr>
                    <td data-label="Name">${d.name}</td>
                    <td data-label="Owner">${ownerName}</td>
                    <td data-label="Topic"><code>${d.mqtt_topic}</code></td>
                    <td data-label="Status"><code>${d.current_state}</code></td>
                    <td data-label="Action">
                        <button class="outline" style="margin:0; padding: 2px 8px;" onclick="toggleDevice('${d.id}', 'admin')">Toggle</button>
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
