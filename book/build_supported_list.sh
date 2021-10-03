#!/bin/sh

find ../service/nbfc_configs/Configs -type f -print0 | xargs -0 -L1 basename -s .xml | sort | sed 's/^/- /g' > GENERATED_MODEL_LIST.md
