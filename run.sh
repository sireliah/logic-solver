#!/bin/bash

set -e

cargo run "$1" && dot -Tsvg graph.dot -o graph.svg
