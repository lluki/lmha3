const app = document.getElementById('app');

async function checkAuth() {
    try {
        console.log('Checking authentication...');
        const resp = await fetch('/api/me');
        if (resp.ok) {
            const user = await resp.json();
            console.log('User authenticated:', user.id);
            renderDashboard(user);
        } else {
            console.log('User not authenticated, rendering login.');
            renderLogin();
        }
    } catch (e) {
        console.error('Auth check failed:', e);
        renderLogin();
    }
}

function renderLogin(error = '') {
    app.innerHTML = `
        <h1>Login</h1>
        ${error ? `<div class="error">${error}</div>` : ''}
        <form class="login-form" id="login-form">
            <input name="username" placeholder="Username" required />
            <input name="password" type="password" placeholder="Password" required />
            <button type="submit">Login</button>
        </form>
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

async function renderDashboard(user) {
    app.innerHTML = `
        <h1>Admin Dashboard</h1>
        <p>Logged in as: ${user.id}</p>
        <div id="dashboard-content">Loading data...</div>
        <br>
        <form id="logout-form">
            <button type="submit">Logout</button>
        </form>
    `;

    document.getElementById('logout-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        await fetch('/api/logout', { method: 'POST' });
        checkAuth();
    });

    loadData();
}

async function loadData() {
    const content = document.getElementById('dashboard-content');
    try {
        console.log('Fetching dashboard data from API...');
        const [tenantsResp, devicesResp] = await Promise.all([
            fetch('/api/tenants'),
            fetch('/api/devices')
        ]);

        if (!tenantsResp.ok || !devicesResp.ok) {
            throw new Error(`API Error: Tenants=${tenantsResp.status}, Devices=${devicesResp.status}`);
        }

        const tenants = await tenantsResp.json();
        const devices = await devicesResp.json();
        
        console.log('Tenants received:', tenants);
        console.log('Devices received:', devices);

        let html = '<h2>Tenants</h2><ul>';
        for (const t of tenants) {
            html += `<li>${t.username || 'Unknown'} (<code>${t.id || 'N/A'}</code>)</li>`;
        }
        html += '</ul>';

        html += '<h2>Devices</h2><table><thead><tr><th>Name</th><th>Owner</th><th>Topic</th><th>Status</th><th>Last Seen</th><th>Action</th></tr></thead><tbody>';
        
        for (const d of devices) {
            console.log('Rendering device:', d);
            let lastSeen = 'Never';
            if (d.last_heartbeat) {
                try {
                    lastSeen = new Date(d.last_heartbeat).toLocaleString();
                } catch (e) {
                    console.warn('Failed to parse date:', d.last_heartbeat);
                    lastSeen = d.last_heartbeat;
                }
            }

            html += `
                <tr>
                    <td>${d.name || 'Unnamed'}</td>
                    <td><code>${d.tenant_id || 'N/A'}</code></td>
                    <td><code>${d.mqtt_topic || 'N/A'}</code></td>
                    <td><strong>${d.current_state || 'UNKNOWN'}</strong></td>
                    <td>${lastSeen}</td>
                    <td>
                        <button onclick="toggleDevice('${d.id}')">Toggle</button>
                    </td>
                </tr>
            `;
        }
        html += '</tbody></table>';
        content.innerHTML = html;
    } catch (e) {
        console.error('LoadData Error:', e);
        content.innerHTML = `<div class="error">Failed to load data: ${e.message}</div>`;
    }
}

window.toggleDevice = async (id) => {
    try {
        const resp = await fetch(`/api/devices/${id}/toggle`, { method: 'POST' });
        if (resp.ok) {
            loadData();
        } else {
            const err = await resp.text();
            alert('Toggle failed: ' + err);
        }
    } catch (e) {
        alert('Toggle failed: ' + e);
    }
};

checkAuth();