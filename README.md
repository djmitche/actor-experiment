/**
 * Thinking:
 *
 *  - an actor _framework_ is hard, esp without dictating an async runtime
 *  - an actor _toolbox_ is different
 *    - pluggable bits: mailboxes, monitoring, telemetry
 *    - patterns: how to write a loop, initialization, ..
 *    - designs: when to use one mailbox or multiple, etc.
 *    - encoded with types, traits, or macros where possible
 *  - shippable as lots of crates, with easy addition by others
 *  - mostly focus on mailboxes
 */


