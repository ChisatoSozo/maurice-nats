import { Link } from "react-router-dom";

export const UserSelectPage = () => {
  const users = ["alice", "bob", "carol"];
  return (
    <>
      {users.map((user) => (
        <Link to={user}>{user}</Link>
      ))}
    </>
  );
};
