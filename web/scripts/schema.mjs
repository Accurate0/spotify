// Writes the graph router's public schema to graph.graphql, which relay-compiler checks against.
// The router only answers introspection for signed-in callers: $GRAPH_TOKEN, or your `ortzi login graph`.
import { execFileSync } from "node:child_process";
import { writeFile } from "node:fs/promises";
import { buildClientSchema, getIntrospectionQuery, printSchema } from "graphql";

const token = process.env.GRAPH_TOKEN ?? execFileSync("ortzi", ["graph", "token"], { encoding: "utf8" }).trim();

const response = await fetch("https://graph.ortzi.cloud/v1/graphql", {
  method: "POST",
  headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
  body: JSON.stringify({ query: getIntrospectionQuery() }),
});
const { data, errors } = await response.json();
if (errors) throw new Error(JSON.stringify(errors));

await writeFile(new URL("../graph.graphql", import.meta.url), `${printSchema(buildClientSchema(data))}\n`);
