#!/usr/bin/env bash
# Test by curl

alerts1='{
    "receiver": "admins",
    "status": "firing",
    "alerts": [
        {
            "status": "firing",
            "labels": {
                "alertname": "something_happend",
                "env": "prod",
                "name": "local-node",
                "instance": "server01.int:9100",
                "job": "node",
                "service": "prometheus_bot",
                "severity": "warning",
                "supervisor": "runit"
            },
            "annotations": {
                "summary": "Oops, something happend!"
            },
            "startsAt": "2016-04-27T20:46:37.903Z",
            "endsAt": "0001-01-01T00:00:00Z",
            "generatorURL": "https://example.com/graph#..."
        }
    ],
    "groupLabels": {
        "alertname": "something_happend",
        "instance": "server01.int:9100"
    },
    "commonLabels": {
        "alertname": "something_happend",
        "env": "prod",
        "instance": "server01.int:9100",
        "job": "node",
        "service": "prometheus_bot",
        "severity": "warning",
        "supervisor": "runit"
    },
    "commonAnnotations": {
        "summary": "runit service prometheus_bot restarted, server01.int:9100"
    },
    "externalURL": "https://alert-manager.example.com",
    "version": "3"
}'


curl http://127.0.0.1:8080/prometheus/hook -i -X POST -H 'Content-Type: application/json' \
    -d "$alerts1"

function add_machine {
    curl http://localhost:8080/machine -i -X POST -H 'Content-Type: application/json' \
        -d '{"targets": ["test.com"], "labels": {"name": "test"}}'
}

function list_machines {
    curl http://localhost:8080/machines
}

function remove_machines {
    curl http://localhost:8080/machine/0 -X DELETE
}

function add_service {
    curl http://localhost:8080/service -i -X POST -H 'Content-Type: application/json' \
        -d '{"targets": ["service.com"], "labels": {"name": "test"}}'
}

function list_services {
    curl http://localhost:8080/services
}

function remove_service {
    curl http://localhost:8080/service/service.com -X DELETE
}
