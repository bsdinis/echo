#!/usr/bin/awk -f

BEGIN {
    n_transfers=0
    sum_transfers=0
    elapsed_sum=0
    total_clients=0
}

/Elapsed/ {
    elapsed_sum += $2
    total_clients += 1
}

/Message Size/ {
    # TODO: check that message sizes are consistent
    message_size=$3
}

/^[0-9]/ {
    if ( n_transfers == 0 ) {
        max_transfer = $1
        min_transfer = $1
    }

    vals[n_transfers] = $1
    n_transfers += 1
    sum_transfers += $1

    if ( max_transfer < $1 ) {
        max_transfer = $1
    }

    if ( min_transfer > $1 ) {
        min_transfer = $1
    }
}


END {
    variance=0
    average=sum_transfers / n_transfers
    for ( i = 0; i < n_transfers ; i++ ) {
        parcel = average - vals[i]
        variance += ( parcel >= 0 ) ? parcel : -parcel
    }

    stddev = sqrt(variance)

    # we average the elapsed times
    # TODO: handle warmup and cooldown
    elapsed = elapsed_sum / total_clients

    printf "N:          %d\n", n_transfers
    printf "Min:        %.3f us\n", min_transfer
    printf "Average:    %.3f us\n", average
    printf "Stddev:     %.3f us\n", stddev
    printf "Max:        %.3f us\n", max_transfer
    printf "Throughput: %.3f B/s\n", (n_transfers * message_size) / elapsed
}
