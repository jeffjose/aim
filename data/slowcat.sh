#!/bin/bash


function slowcat(){ while read; do sleep .05; echo "$REPLY"; done; }
