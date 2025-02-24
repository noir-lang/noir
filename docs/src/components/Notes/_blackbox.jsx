import Link from '@docusaurus/Link';

export default function BlackBoxInfo({ to }) {
  return (
    <div>
      <p>
        This is a black box function. Read <Link to={to}>this section</Link> to learn more about black box functions in
        Noir.
      </p>
    </div>
  );
}
