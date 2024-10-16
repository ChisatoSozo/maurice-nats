import { Vector3 } from "@babylonjs/core";
import { Engine, Scene } from "react-babylonjs";
import { feetAndInches } from "../utils/util";
import { useEffect } from "react";
import { HouseRenderer } from "../house/HouseRenderer";

export const HouseViewerPage = () => {
  useEffect(() => {
    const canvas = document.getElementById("babylonJS");
    if (canvas === null) {
      return;
    }
    const parent = canvas.parentElement;
    if (parent === null) {
      return;
    }
    const cs = getComputedStyle(parent);

    const paddingX = parseFloat(cs.paddingLeft) + parseFloat(cs.paddingRight);
    const paddingY = parseFloat(cs.paddingTop) + parseFloat(cs.paddingBottom);

    // Element width and height minus padding and border
    const elementWidth = parent.clientWidth - paddingX;
    const elementHeight = parent.clientHeight - paddingY - 8;
    console.log(elementWidth, elementHeight);
    canvas.style.width = elementWidth + "px";
    canvas.style.height = elementHeight + "px";
  }, []);

  return (
    <Engine
      style={{
        height: "100%",
        width: "100%",
      }}
      antialias
      adaptToDeviceRatio
      canvasId="babylonJS"
    >
      <Scene>
        <arcRotateCamera
          name="camera1"
          target={Vector3.Zero()}
          alpha={Math.PI / 2}
          beta={Math.PI / 4}
          radius={feetAndInches(50)}
          wheelPrecision={0.4}
          panningSensibility={10}
        />
        <hemisphericLight
          name="light1"
          intensity={0.7}
          direction={Vector3.Up()}
        />
        <HouseRenderer />
      </Scene>
    </Engine>
  );
};
