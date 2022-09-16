#!/bin/bash

kill -9 $(ps aux | grep juno | grep -v grep | awk -v x=2 '{print $x}')
