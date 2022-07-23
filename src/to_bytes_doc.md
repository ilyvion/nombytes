Returns a [`Bytes`] representing the real state of the inner one.

Be careful using a [`Bytes`] returned from this function to create
a new [`NomBytes`] for use in *the same parsing session*, as due to
an optimization in [`Bytes`], creating an empty slice (e.g. asking
for the slice of `0..0` or `..0`, which [`nom`] sometimes does)
results in a [`Bytes`] that is unrelated to its source, which
causes later offset calculations to give incorrect results.

This behavior is accounted for internally, so as long as you stick
to only using [`NomBytes`] directly without going to [`Bytes`] and back,
you won't be affected by this optimization behavior.
