import paho.mqtt.client as mqtt
import os
import time

host = os.environ.get("LMHA_MQTT_HOST", "localhost")
port = int(os.environ.get("LMHA_MQTT_PORT", 1883))
user = os.environ.get("LMHA_MQTT_USER", "admin")
password = os.environ.get("LMHA_MQTT_PASSWORD", "changeme")

client = mqtt.Client()
client.username_pw_set(user, password)

topics = []

def on_message(client, userdata, msg):
    print(f"Found topic: {msg.topic}")
    topics.append(msg.topic)

client.on_message = on_message
client.connect(host, port)
client.subscribe("lmha3/instances/#")

print("Scanning for topics...")
client.loop_start()
time.sleep(2)
client.loop_stop()

print(f"Clearing {len(topics)} topics...")
for topic in topics:
    print(f"Clearing {topic}")
    client.publish(topic, "", retain=True)

client.disconnect()
print("Done.")
