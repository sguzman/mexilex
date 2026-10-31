import test from "node:test";
import assert from "node:assert/strict";
import { fold, bucketOf, resultMatches, rankMexicanFrequency } from "../site/search.mjs";

test("accent-insensitive Spanish lookup with exact source spellings unchanged", () => {
  assert.equal(fold("ÁRBOL"), "arbol");
  assert.equal(fold("NIÑO"), "niño");
  assert.equal(fold("niño"), "niño");
  assert.notEqual(fold("niño"), fold("nino"));
  assert.equal(bucketOf("Árbol"), "ar");
  assert.equal(bucketOf("ñame"), "_a");
  assert.equal(bucketOf("a"), null);
  assert.equal(bucketOf(""), null);
});
test("prefix matching preserves Mexican labels as independently selectable", () => {
  const mexican = ["banqueta","a".repeat(64),"noun","sidewalk",true,false];
  const regular = ["barco","b".repeat(64),"noun","boat",false,false];
  assert.equal(resultMatches(mexican,"BAN",true),true);
  assert.equal(resultMatches(regular,"ba",true),false);
  assert.equal(resultMatches(regular,"ba",false),true);
  assert.equal(resultMatches(mexican,"bañ"),false);
});
test("inflected forms require explicit opt-in", () => {
  const inflected = ["caminé","c".repeat(64),"verb","preterite of caminar",false,true];
  assert.equal(resultMatches(inflected,"cami"),false);
  assert.equal(resultMatches(inflected,"CAMI",false,true),true);
  assert.equal(resultMatches(inflected,"cami",true,true),false);
});

test("Mexican dialogue evidence sorts safely without confusing regional labels", () => {
  const a = ["casa","a","noun","house",false,false,[61,42,43,10]];
  const b = ["chamba","b","noun","work",true,false,[5,0,1,4]];
  const c = ["banqueta","c","noun","sidewalk",true,false,null];
  assert.deepEqual([c,b,a].sort(rankMexicanFrequency), [a,b,c]);
  assert.equal(a[4], false);
  assert.equal(b[4], true);
});
