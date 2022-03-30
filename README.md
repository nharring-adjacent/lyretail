# lyretail
Lyretail is a streaming text parser and categorizer based on the "Drain" algorithm.

Right now this is barely past the proof of concept stage, it was initially written entirely to provide a test-harness for using the `drain-flow` crate which does the actual algorithm implementation and took this form since I have occasionally wished for this tool when not working in places which have their own version. 

## What it does
lyretail consumes a stream of input lines, tokenizes them and then processes them according to the Drain algorithm as implemented in the crate [drain-flow](https://github.com/nharring-adjacent/drain-flow). Periodically lyretail will output meta-data about the lines it has processed and the buckets they have sorted into. This stream can either be a file supplied with the `--file=<Path>` option or from stdin which is the default.

When reading a static file a summary of the process will be printed to `stdout` including all discovered events and how many times they matched, when reading from stdin this will occur when you hit ctrl-c. In both cases it is also possible to have this output printed periodically by specifying `--periodic` and optionally picking an interval with `--interval`.

## Why would I use it?
Say you've got a busy instance of a service writing a huge amount of log data and you want to get a general idea of what its logging about. Unless you're really lucky there probably isn't a consistent format to the output, and most tools like logstash required at least some operator guidance on what patterns to apply.
Lyretail is different, requiring no upfront knowledge of stream contents and streadily sifting out the constant portions of log messages from the variable parts. 

## Example usage
This recording was generated using the demo.sh with no arguments on a Macbook air which has `/var/log/wifi.log`.
[![asciicast](https://asciinema.org/a/481881.svg)](https://asciinema.org/a/481881)

## What's with the name?
[Lyretail Coralfish](https://en.wikipedia.org/wiki/Sea_goldie) are members of the grouper (and sea bass!) family, and in my mind
that's what this tool does: it groups lines! There are many groupers, but this is the only one with `tail` in its name which seemed
too fitting not to use!