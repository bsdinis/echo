#!/usr/bin/env zsh

set -xe

GROUP=echo
CLIENTS=echo-clients
SERVER=node0

TINY=1
SMALL=4K
MEDIUM=256K
HUGE=1M

# generate the logname for an experiment
# should be placed in /tmp
function logname() {
    impl=$1
    obj_type=$2
    echo "${1}_${2}.log"
}

# get the logs from an experiment
# logs are stored in logs/${size}/${impl}_${client}.log
function get_logs() {
    impl=$1
    obj_type=$2

    logfile=$(logname ${impl} ${obj_type})
    for client in $(cat "${HOME}/.dsh/group/${CLIENTS}");
    do
        dir=${ECHO_HOME}/logs/${obj_type}
        mkdir -p ${dir}
        scp "${client}":"/tmp/${logfile}" "${dir}/${impl}_${client}.log"
    done
}

# get object size from type (tiny|small|medium|huge)
function get_obj_size() {
    case $1 in
        tiny) obj_size=$TINY;;
        small) obj_size=$SMALL;;
        medium) obj_size=$MEDIUM;;
        huge) obj_size=$HUGE;;
        *)
            echo >&2 "error: unknown size descriptor"
            return 255
        ;;
    esac
    echo ${obj_size}
}

function run_python() {
    ssh $SERVER "killall python" || true

    for obj_type in $@;
    do
        ssh $SERVER "cd dev/echo/python && source venv/bin/activate && python server.py $SERVER" &
        sleep 5
        logfile=$(logname "python" "${obj_type}")
        obj_size=$(get_obj_size "${obj_type}")

        dsh -c -g $CLIENTS "cd dev/echo/python && source venv/bin/activate && python client.py $SERVER --message-size ${obj_size} > /tmp/${logfile}"

        ssh $SERVER "killall python"
        get_logs "python" "${obj_type}"
    done
}

function run_python_grpc() {
    ssh $SERVER "killall python" || true

    for obj_type in $@;
    do
        ssh $SERVER "cd dev/echo/python_grpc && source venv/bin/activate && python server.py $SERVER" &
        sleep 5
        logfile=$(logname "python_grpc" "${obj_type}")
        obj_size=$(get_obj_size "${obj_type}")

        dsh -c -g $CLIENTS "cd dev/echo/python_grpc && source venv/bin/activate && python client.py $SERVER --message-size ${obj_size} > /tmp/${logfile}"

        ssh $SERVER "killall python"
        get_logs "python_grpc" "${obj_type}"
    done
}

function run_python_async_grpc() {
    ssh $SERVER "killall python" || true

    for obj_type in $@;
    do
        ssh $SERVER "cd dev/echo/python_async_grpc && source venv/bin/activate && python server.py $SERVER" &
        sleep 5
        logfile=$(logname "python_async_grpc" "${obj_type}")
        obj_size=$(get_obj_size "${obj_type}")

        dsh -c -g $CLIENTS "cd dev/echo/python_async_grpc && source venv/bin/activate && python client.py $SERVER --message-size ${obj_size} > /tmp/${logfile}"

        ssh $SERVER "killall python"
        get_logs "python_async_grpc" "${obj_type}"
    done
}

function run_rust_sync() {
    ssh $SERVER "killall server" || true

    for obj_type in $@;
    do
        ssh $SERVER "dev/echo/rust_sync/target/release/server  $SERVER" &
        sleep 5
        logfile=$(logname "rust_sync" "${obj_type}")
        obj_size=$(get_obj_size "${obj_type}")

        dsh -c -g $CLIENTS "dev/echo/rust_sync/target/release/client $SERVER --message-size ${obj_size} > /tmp/${logfile}"

        ssh $SERVER "killall server"
        get_logs "rust_sync" "${obj_type}"
    done
}

function run_rust_async() {
    ssh $SERVER "killall server" || true

    for client_type in closed bursty;
    do
        for obj_type in $@;
        do
            ssh $SERVER "dev/echo/rust_async/target/release/server  $SERVER" &
            sleep 5
            logfile=$(logname "rust_async_${client_type}" "${obj_type}")
            obj_size=$(get_obj_size "${obj_type}")

            dsh -c -g $CLIENTS "dev/echo/rust_async/target/release/client $SERVER --message-size ${obj_size} --client-type ${client_type} > /tmp/${logfile}" || true

            ssh $SERVER "killall server"
            get_logs "rust_async_${client_type}" "${obj_type}"
        done
    done
}

function run_rust_tonic() {
    ssh $SERVER "killall server" || true

    for client_type in closed bursty;
    do
        for obj_type in $@;
        do
            ssh $SERVER "dev/echo/rust_tonic/target/release/server  $SERVER" &
            sleep 5
            logfile=$(logname "rust_tonic_${client_type}" "${obj_type}")
            obj_size=$(get_obj_size "${obj_type}")

            dsh -c -g $CLIENTS "dev/echo/rust_tonic/target/release/client $SERVER --message-size ${obj_size} --client-type ${client_type} > /tmp/${logfile}" || true

            ssh $SERVER "killall server"
            get_logs "rust_tonic_${client_type}" "${obj_type}"
        done
    done
}

run_python $@
run_python_grpc $@
run_python_async_grpc $@
run_rust_sync $@
run_rust_async $@
run_rust_tonic $@
