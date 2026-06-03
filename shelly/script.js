let CONFIG = {
  start: 1, timeout: 3600, interval: 1800000 // 30 min
};

let disconnect_time = null;

Timer.set(CONFIG.interval, true, function() {
  let mqtt = Shelly.getComponentStatus("mqtt");
  let sys = Shelly.getComponentStatus("sys");

  if (mqtt && mqtt.connected) {
    disconnect_time = null;
    return;
  }

  if (disconnect_time === null) disconnect_time = sys.uptime;

  // If offline for more than 1 hour
  if ((sys.uptime - disconnect_time) > CONFIG.timeout) {
    // Direct string split to get the hour from "HH:MM"
    let hour = JSON.parse(sys.time.split(":")[0]);
    let should_be_on = (hour >= 1 && hour < 6);
    
    print("Failsafe active. Hour:", hour, "Setting:", should_be_on);
    Shelly.call("Switch.Set", { id: 0, on: should_be_on });
  }
});
