#!/bin/sh
for po in po/*.po; do
    msgmerge --update "$po" po/fcitx5-lotus.pot;
done