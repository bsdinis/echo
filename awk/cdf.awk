#!/usr/bin/awk -f

BEGIN {
    n_transfers=0
}

/^[0-9]/ {
    n_transfers += 1
    vals[n_transfers] = $1
}


END {
    asort(vals)

    for ( i = 1; i <= n_transfers ; i++ ) {
        printf "%.3f %.3f\n", 100 * (i / n_transfers), vals[i]
    }
}
