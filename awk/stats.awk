#!/usr/bin/awk -f

function compute_avg_elapsed(start_p, end_p) {
    elapsed_count = 0
    elapsed_sum = 0
    for (id in start_p) {
        if (id in end_p) {
            elapsed_sum += end_p[id] - start_p[id]
            elapsed_count += 1
        }
    }

    return elapsed_sum / elapsed_count
}

function n_clients(start_p, end_p) {
    count = 0
    for (id in start_p) {
        if (id in end_p) {
            count += 1
        }
    }

    return count
}

BEGIN {
    n_transfers=0
    sum_transfers=0
    elapsed_sum=0
}

/^Start:/ {
    id = $2
    ts = $3
    if ( id in start ) {
        if ( ts < start[id] ) {
            start[id] = ts
        }
    }
    else {
        start[id] = ts
    }
}

/^End:/ {
    id = $2
    ts = $3
    if ( id in end ) {
        if ( ts > end[id] ) {
            end[id] = ts
        }
    }
    else {
        end[id] = ts
    }
}

/^Message Size:/ {
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

    elapsed = compute_avg_elapsed(start, end)
    n_cli = n_clients(start, end)

    printf "#Transfers:  %d\n", n_transfers
    printf "Min:         %.3f us\n", min_transfer
    printf "Average:     %.3f us\n", average
    printf "Stddev:      %.3f us\n", stddev
    printf "Max:         %.3f us\n", max_transfer
    printf "Throughput:  %.3f B/s\n", (n_transfers * message_size) / elapsed
    printf "Elapsed Avg: %.9f s\n", elapsed
    printf "#Clients:    %d\n", n_cli
}
