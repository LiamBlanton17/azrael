#!/usr/bin/env bash
set -euo pipefail

./fastchess/fastchess \
    -engine cmd=./azrael_feature name=feature \
    -engine cmd=./azrael_main    name=main \
    -each proto=uci tc=30+1 \
    -openings file=./openings.epd format=epd order=random \
    -rounds 40000 -games 2 -repeat \
    -concurrency 3 \
    -sprt elo0=0 elo1=10 alpha=0.035 beta=0.035 model=normalized \
    -resign movecount=3 score=600 \
    -draw movenumber=40 movecount=8 score=10 \
    -recover \
    -ratinginterval 250 -report penta=true \
    -config file=config.json \
    -pgnout file=out.pgn \
    -log file=fc.log level=warn
    