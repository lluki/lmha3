import requests
import json
from datetime import datetime

# 1. Define your physical system configuration
CONFIG = {
    "lat": "47.44",      # Example latitude
    "lon": "8.56",       # Example longitude
    "dec": "35",         # Panel tilt/declination (35 degrees)
    "az": "20",           # Facing due South (0 degrees)
    "kwp": "16"         # 5.5 kWp installed capacity
}

# 2. Construct the Forecast.Solar public API URL
url = f"https://api.forecast.solar/estimate/{CONFIG['lat']}/{CONFIG['lon']}/{CONFIG['dec']}/{CONFIG['az']}/{CONFIG['kwp']}"

headers = {
    "Accept": "application/json",
    "User-Agent": "HomeEnergyManagerScript/1.0" # Good practice to identify your script
}

try:
    print("Fetching PV forecast data...")
    response = requests.get(url, headers=headers, timeout=10)
    
    # Handle API rate limits or errors gracefully
    if response.status_code == 429:
        print("Rate limit reached! (Free tier allows 12 requests per hour). Try again later.")
    elif response.status_code != 200:
        print(f"Failed to fetch data. Status code: {response.status_code}")
        print(response.text)
    else:
        data = response.json()
        
        print("\n=== Full JSON Response ===")
        print(json.dumps(data, indent=4))

        result = data.get("result", {})
        
        # 3. Parse and display summary data
        print("\n=== Daily Production Forecasts ===")
        totals = result.get("watt_hours_day", {})
        for date, wh in totals.items():
            kwh = wh / 1000.0
            print(f"Date: {date} | Estimated Total: {kwh:.2f} kWh")
            
        print("\n=== Upcoming Hourly Power Output (Watts) ===")
        hourly_watts = result.get("watts", {})
        
        # Only print the next few hours to keep the terminal clean
        now = datetime.now()
        count = 0
        for timestamp, watts in hourly_watts.items():
            dt = datetime.strptime(timestamp, "%Y-%m-%d %H:%M:%S")
            print(f"Time: {dt.strftime('%H:%M')} | Expected Power: {watts} W")
            count += 1

except requests.exceptions.RequestException as e:
    print(f"An error occurred connecting to the API: {e}")
