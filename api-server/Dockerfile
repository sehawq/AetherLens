# Build Stage
FROM mcr.microsoft.com/dotnet/sdk:8.0 AS build
WORKDIR /src

# Copy csproj and restore
COPY ["apex-backend.csproj", "./"]
RUN dotnet restore "apex-backend.csproj"

# Copy everything else and build
COPY . .
RUN dotnet publish "apex-backend.csproj" -c Release -o /app/publish

# Runtime Stage
FROM mcr.microsoft.com/dotnet/aspnet:8.0
WORKDIR /app
COPY --from=build /app/publish .

# Expose ports
EXPOSE 5000

# Set environment variables
ENV ASPNETCORE_URLS=http://+:5000
ENV ConnectionStrings__LocalDb="Data Source=guardian.db"

ENTRYPOINT ["dotnet", "apex-backend.dll"]
