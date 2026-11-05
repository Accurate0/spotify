// Writes the graph router's public schema to graph.graphql, which relay-compiler checks against.
import { writeFile } from "node:fs/promises";
import { buildClientSchema, getIntrospectionQuery, printSchema } from "graphql";

const response = await fetch("https://graph.ortzi.cloud/v1/graphql", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ query: getIntrospectionQuery() }),
});
const { data, errors } = await response.json();
if (errors) throw new Error(JSON.stringify(errors));

await writeFile(new URL("../graph.graphql", import.meta.url), `${printSchema(buildClientSchema(data))}\n`);
