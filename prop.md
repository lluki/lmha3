# Support multiple houses

We want to support multiple houses. 
Each house has one PV. Each tenant belongs to a house. implicitly, a device belongs to a house via the tenant. Move the configuration of the PV (HA token, HA address etc.) into the admin panel. The UI will change, if a normal user logs in, the house of the tenant is preselected. If an admin user logs in (admins have global visiblity ) a dropdown on top will appear to chose the house. There will only be a handful of houses. 

Few UI improvements:
* Make sure the selected house is clearly stated in the overview above the PV production as header. 
 * The same in the admin panel. 
 * Make sure to reload the contents if another house is chosen. 
 * In the history, if the box "Show All Telemetry" is unchecked, then the list is truncated too much. I think we need another server roundtrip and have it filter on the state events= otherwise the table cuts out way too early. 
 * In the admin panel, we dont need the systems log. We already have it in Logs
 * In the admin panel, make it possible to create users. Also delete, BUT ONLY if they have no devices.
 * If we add a shelly, it would be really cool to fetch from the logs shelly ids. in case one has connected that has not been registered. make this as a suggestion dropdown, but still allow manually set values.
 * If i click manually on toggle, i want the content to be reloaded (most likely state change sfrom ON->OFF or vice versa)/

Verification: Run testcase, i will manually look at the webui.

In the overview, we now show the last-on period. I think it would be better to show how long the boiler was on in the last 24h period (starting from 5am current day). 






































--> all file open mode. 
--> Running 8xH100
--> VAST dynamo might be other option. 

--> Chekc what is GUSLI ?
