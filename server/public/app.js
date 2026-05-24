const app = document.getElementById('app');
const userInfo = document.getElementById('user-info');
const mainNav = document.getElementById('main-nav');
const adminTab = document.getElementById('admin-tab');

let currentUser = null;
let activeTab = 'overview';

async function checkAuth() {
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
            <header><strong>Your Devices</strong></header>
            <div id="overview-content" aria-busy="true">Loading...</div>
        </article>
    `;
    
    try {
        const resp = await fetch('/api/devices');
        if (!resp.ok) throw new Error('Failed to load devices');
        const devices = await resp.json();
        
        const userDevices = devices.filter(d => d.tenant_id === currentUser.tenant_id);
        const content = document.getElementById('overview-content');
        content.removeAttribute('aria-busy');

        if (userDevices.length === 0) {
            content.innerHTML = '<p>No devices found.</p>';
            return;
        }

        let html = '<table><thead><tr><th>Name</th><th>Status</th><th>Last Seen</th><th>Action</th></tr></thead><tbody>';
        for (const d of userDevices) {
            let lastSeen = d.last_heartbeat ? new Date(d.last_heartbeat).toLocaleString() : 'Never';
            html += `
                <tr>
                    <td data-label="Name">${d.name}</td>
                    <td data-label="Status"><code>${d.current_state}</code></td>
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
        content.removeAttribute('aria-busy');
        content.innerHTML = `<p style="color: red;">${e.message}</p>`;
    }
}

function renderHistory() {
    app.innerHTML = `
        <article>
            <header><strong>Consumption History</strong></header>
            <p>Historical data visualization is coming soon.</p>
        </article>
    `;
}

async function renderAdmin() {
    app.innerHTML = `
        <article>
            <header><strong>Admin: Tenants</strong></header>
            <div id="admin-tenants" aria-busy="true">Loading...</div>
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

        // Tenants
        const tenantsDiv = document.getElementById('admin-tenants');
        tenantsDiv.removeAttribute('aria-busy');
        let tenantsHtml = '<ul>';
        for (const t of tenants) {
            tenantsHtml += `<li>${t.username} <small>(<code>${t.id}</code>)</small></li>`;
        }
        tenantsHtml += '</ul>';
        tenantsDiv.innerHTML = tenantsHtml;

        // Devices
        const devicesDiv = document.getElementById('admin-devices');
        devicesDiv.removeAttribute('aria-busy');
        let devicesHtml = '<table><thead><tr><th>Name</th><th>Owner</th><th>Topic</th><th>Status</th><th>Action</th></tr></thead><tbody>';
        for (const d of devices) {
            devicesHtml += `
                <tr>
                    <td data-label="Name">${d.name}</td>
                    <td data-label="Owner"><code>${d.tenant_id.split('-')[0]}...</code></td>
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
