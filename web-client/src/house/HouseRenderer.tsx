import { Color3, Vector3 } from "@babylonjs/core";
import {
  houseConstants,
  HouseElement,
  Layer,
} from "../constants/houseConstants";

export const HouseRenderer = () => (
  <>
    {Object.keys(houseConstants).map((layerName) => (
      <LayerRenderer key={layerName} layer={houseConstants[layerName]} />
    ))}
  </>
);

export const LayerRenderer = ({ layer }: { layer: Layer }) => (
  <>
    {Object.keys(layer).map((elementName, i) => (
      <ElementRenderer
        key={elementName}
        index={i}
        element={layer[elementName]}
      />
    ))}
  </>
);

export const ElementRenderer = ({
  element,
  index,
}: {
  element: HouseElement;
  index: number;
}) => {
  switch (element.type) {
    case "sitePlan":
      return (
        <ground
          name=""
          position={new Vector3(element.x, index, element.y)}
          width={element.width}
          height={element.depth}
        >
          <standardMaterial
            name="groundMaterial"
            diffuseColor={Color3.Random()}
          />
        </ground>
      );
    default:
      throw new Error(`Unknown element type: ${element.type}`);
  }
};
