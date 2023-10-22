#!/usr/bin/env bash

# Gotta workaround the genius idea of naming multiple packages the same thing
# https://github.com/colour-science/colour/issues/958

# TODO query python version.

rm -rf venv
python -m venv venv
source ./venv/bin/activate
pip install -r requirements.txt
mv venv/lib/python3.11/site-packages/colour.py venv/lib/python3.11/site-packages/no_collision_colour.py
pip install colour-science==0.4.3
echo "from no_collision_colour import *" >> venv/lib/python3.11/site-packages/colour/__init__.py

