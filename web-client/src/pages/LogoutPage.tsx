import { useEffect } from "react";
import { useNavigate } from "react-router-dom";

export const LogoutPage = () => {
  const navigate = useNavigate();
  useEffect(() => {
    navigate("/");
  }, [navigate]);

  return (
    <div>
      <h1>Logout</h1>
    </div>
  );
};
