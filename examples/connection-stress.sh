#!/bin/bash

cargo r -p wsio-client --example connection_stress --all-features "${@}"
