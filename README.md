Thinking:

 * an actor _framework_ is hard, esp without dictating an async runtime
 * an actor _toolbox_ is different
   * pluggable bits: mailboxes, monitoring, telemetry
   * patterns: how to write a loop, initialization, ..
   * designs: when to use one mailbox or multiple, etc.
   * encoded with types, traits, or macros where possible
 * shippable as lots of crates, with easy addition by others
 * mostly focus on mailboxes

TODO:
 * timer mailbox
 * watch-based "condition" mailbox (can await to see new values)
 * define actor spawn as Type { .. }.spawn
 * stream mailbox
 * canned actor to map over a stream, later in parallel

Pipeline TODO:
 * Commitment trait? mailbox?
 * Need more stages to be interesting?
 * switch to Bytes

Polish TODO:
 * use a better error wrapper that doesn't require Debug on T everywhere
