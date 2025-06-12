namespace VisitorDemo;

public class ContactsMetadata(int version) : IType<Contacts>, IStructType<Contacts>
{
    public void Accept<V>(V visitor) where V : ITypeVisitor<Contacts>, allows ref struct => visitor.VisitStruct(Getters.Identity, this);

    void IStructType<Contacts>.Accept<V>(V visitor)
    {
        visitor.VisitField("Email", (ref Contacts s) => ref s.Email, StringMetadata.Instance);
        visitor.VisitField("PhoneNumber", (ref Contacts s) => ref s.PhoneNumber, StringMetadata.Instance);
    }
}
