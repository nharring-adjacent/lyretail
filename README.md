# lyretail
Lyretail is a streaming text parser and categorizer based on the "Drain" algorithm. 


## What it does
lyretail consumes a stream of input lines, tokenizes them and then processes them according to the Drain algorithm as implemented in the crate [drain-flow](https://github.com/nharring-adjacent/drain-flow). Periodically lyretail will output meta-data about the lines it has processed and the buckets they have sorted into.

## Why would I use it?
Say you've got a busy instance of a service writing a huge amount of log data and you want to get a general idea of what its logging about. Unless you're really lucky there probably isn't a consistent format to the output, and most tools like logstash required at least some operator guidance on what patterns to apply.
Lyretail is different, requiring no upfront knowledge of stream contents and streadily sifting out the constant portions of log messages from the variable parts. 

## Example output
An example on a Macbook doing `cat /var/log/wifi.log|cut -d ' ' -f5-|lyretail`:
```
LogGroup ID: 26fnfLvrKu8Vki3szOMYHhlg5X1
First Seen: 2022-03-21 00:46:49 UTC
Event: <airport[590]> systemWokenByWiFi: System Wake Reason not found

233 examples and 0 wildcards

LogGroup ID: 26fnfGJiuW9uY2bRXDcqbUhiCSD
First Seen: 2022-03-21 00:46:49 UTC
Event: Apple80211Set:10289 Processing APPLE80211_IOC_ROAM

3 examples and 0 wildcards

LogGroup ID: 26fnfFN5ukqfqMu0VqpqY174wdk
First Seen: 2022-03-21 00:46:49 UTC
Event: <airport[590]> _configureTCPKeepAlive: Unable to enable TCP keep-alive on en0 (Operation not supported)

176 examples and 0 wildcards

LogGroup ID: 26fnfHgrpfPPXLDfNYccgxY4lFF
First Seen: 2022-03-21 00:46:49 UTC
Event: <airport[590]> airportdProcessDLILEvent: en0 marked UP, applying user power preferences (ON).

3 examples and 0 wildcards

LogGroup ID: 26fnfJtHFWYryALg0woMFh96Rdu
First Seen: 2022-03-21 00:46:49 UTC
Event: Usb Host Notification Apple80211Set: seqNum 130 Total 1 chg 1 en0

258 examples and 0 wildcards

LogGroup ID: 26fnfFjFtFdggBOy8pf6hHHStex
First Seen: 2022-03-21 00:46:49 UTC
Event: newsyslog[46185]: logfile turned over

0 examples and 0 wildcards

LogGroup ID: 26fnfG4yvPcoOe8KmM3jGWVRhGK
First Seen: 2022-03-21 00:46:49 UTC
Event: Apple80211Set:10303 Processing APPLE80211_IOC_ROAM dataRef:0x15a728b10

3 examples and 0 wildcards

LogGroup ID: 26fnfHQiDHr4uZiLVjHVSqrC3bV
First Seen: 2022-03-21 00:46:49 UTC
Event: Apple80211Set:10324 CFType is CFData

3 examples and 0 wildcards

LogGroup ID: 26fnfJCZAGDJH6MKjvud5Y4rNU8
First Seen: 2022-03-21 00:46:49 UTC
Event: <airport[590]> ERROR: rapportd (951) is not entitled for com.apple.wifi.join_history, will not allow request

179 examples and 0 wildcards

LogGroup ID: 26fnfGRzOwSyd2RdSeCfBzwcvAw
First Seen: 2022-03-21 00:46:49 UTC
Event: <airport[590]> airportdProcessDLILEvent: <en0> event=1, flags=0xFFFF8863, lastPowerPref=on

7 examples and 0 wildcards
```

## What's with the name?
[Lyretail Coralfish](https://en.wikipedia.org/wiki/Sea_goldie) are members of the grouper (and sea bass!) family, and in my mind
that's what this tool does: it groups lines! There are many groupers, but this is the only one with `tail` in its name which seemed
too fitting not to use!