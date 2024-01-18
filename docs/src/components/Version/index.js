import React from "react";
import { usePluginData } from "@docusaurus/useGlobalData";

const Versions = () => usePluginData("load-versions").versions;

export default function Version({ what }) {
  return Versions()[what];
}

export const AztecPackagesVersion = () => Versions()["aztec-packages"];