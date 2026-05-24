const app = document.getElementById('app');

async function checkAuth() {
    try {
        const resp = await fetch('/api/me');
        if (resp.ok) {
            const user = await resp.json();
            renderDashboard(user);
        } else {
            renderLogin();
        }
    } catch (e) {
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
        const data = Object.from_slice(Array.from(formData.entries())); // Simple form to object
        // Wait, Object.from_slice? No, Object.fromEntries.
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
        const [tenantsResp, devicesResp] = await Promise.all([
            fetch('/api/tenants'),
            fetch('/api/devices')
        ]);

        const tenants = await tenantsResp.json();
        const devices = await devicesResp.json();

        let html = '<h2>Tenants</h2><ul>';
        tenants.forEach(t => {
            html += `<li>${t.username} (${t.id})</li>`;
        });
        html += '</ul><h2>Devices</h2><table><tr><th>Name</th><th>Owner</th><th>Topic</th><th>Status</th><th>Last Seen</th><th>Action</th></tr>';
        
        devices.forEach(d => {
            const lastSeen = d.last_heartbeat ? new Date(d.last_heartbeat).toLocaleString() : 'Never';
            html += `
                <tr>
                    <td>${d.name}</td>
                    <td>${d.tenant_id}</td>
                    <td>${d.mqtt_topic}</td>
                    <td>${d.current_state}</td>
                    <td>${last_seen}</td>
                    <td>
                        <button onclick="toggleDevice('${d.id}')">Toggle</button>
                    </td>
                </tr>
            `;
        });
        html += '</table>';
        content.innerHTML = html;
    } catch (e) {
        content.innerHTML = '<div class="error">Failed to load data</div>';
    }
}

window.toggleDevice = async (id) => {
    try {
        const resp = await fetch(`/api/devices/${id}/toggle`, { method: 'POST' });
        if (resp.ok) {
            loadData();
        } else {
            alert('Toggle failed');
        }
    } catch (e) {
        alert('Toggle failed');
    }
};

checkAuth();