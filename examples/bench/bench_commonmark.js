#!/usr/bin/env node
"use strict";

var fs = require("fs");
var commonmark = require("commonmark");

var parser = new commonmark.Parser();
var renderer = new commonmark.HtmlRenderer();

var benchfile = process.argv[2];
var contents = fs.readFileSync(benchfile, "utf8");

// Run once (hyperfine will handle iterations)
renderer.render(parser.parse(contents));
