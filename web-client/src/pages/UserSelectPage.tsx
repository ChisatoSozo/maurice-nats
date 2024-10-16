import { Link } from "react-router-dom";

export const UserSelectPage = () => {
  const users = ["sara", "jodie"];
  return (
    <ul>
      {users.map((user) => (
        <li key={user}>
          <Link to={user}>{user}</Link>
        </li>
      ))}
    </ul>
  );
};
